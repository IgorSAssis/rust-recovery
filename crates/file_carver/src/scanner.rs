use std::collections::{HashMap, HashSet};
use std::io::{self, Read, Seek, SeekFrom};

use crate::carved_file::CarvedFile;
use crate::matcher::PatternMatcher;
use crate::signature::{FileKind, Signature};
use crate::window::{ScanWindow, WindowSlice};

/// A file whose header has been found but whose footer has not yet been located.
struct PendingFile {
    kind: FileKind,
    start_abs: u64,
}

/// Scans a byte source for files using binary signature detection
/// (header + footer patterns).
///
/// # Usage
///
/// ```ignore
/// let carved = Scanner::new()
///     .add_signature(&JPEG_SIGNATURE)
///     .add_signature(&PNG_SIGNATURE)
///     .with_chunk_size(4096)
///     .scan(&mut source)?;
/// ```
pub struct Scanner {
    chunk_size: usize,
    signatures: Vec<&'static Signature>,
}

impl Default for Scanner {
    fn default() -> Self {
        Self {
            chunk_size: 4096,
            signatures: Vec::new(),
        }
    }
}

impl Scanner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the number of bytes read from the source per iteration.
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Registers a file signature to be detected during scanning.
    pub fn add_signature(mut self, signature: &'static Signature) -> Self {
        self.signatures.push(signature);
        self
    }

    /// Scans `source` from the beginning and returns all detected files,
    /// sorted by their starting offset.
    pub fn scan<R: Read + Seek>(&self, source: &mut R) -> io::Result<Vec<CarvedFile>> {
        if self.signatures.is_empty() || self.chunk_size == 0 {
            return Ok(Vec::new());
        }

        source.seek(SeekFrom::Start(0))?;

        let overlap_size = self.compute_overlap_size();
        let mut scan_window = ScanWindow::new(overlap_size, self.chunk_size);

        let mut carved: Vec<CarvedFile> = Vec::new();
        let mut pending: Vec<PendingFile> = Vec::new();
        // Tracks absolute offsets of already-registered headers to prevent the
        // overlap region from producing duplicates on the next iteration.
        let mut known_headers: HashSet<u64> = HashSet::new();

        while let Some(window) = scan_window.next(source)? {
            let (still_open, closed) = self.close_pending(pending, &window);
            let (new_pending, found, new_headers) = self.find_new_headers(&window, &known_headers);

            carved.extend(closed);
            carved.extend(found);
            pending = still_open;
            pending.extend(new_pending);
            known_headers.extend(new_headers);
        }

        carved.sort_by_key(|carved_file| carved_file.offset_start);
        Ok(carved)
    }

    // ── private helpers ───────────────────────────────────────────────────────

    /// Attempts to close each pending file by searching for its footer in the
    /// current window.
    ///
    /// Returns a tuple `(still_open, closed)`:
    /// - `still_open`: files whose footer was not found yet.
    /// - `closed`: files that were successfully closed in this window.
    fn close_pending(
        &self,
        pending: Vec<PendingFile>,
        window: &WindowSlice,
    ) -> (Vec<PendingFile>, Vec<CarvedFile>) {
        // Per-kind cursor: when multiple files of the same type are open,
        // footers are consumed in FIFO order so that each header gets
        // matched to the correct footer.
        let mut footer_cursors: HashMap<FileKind, usize> = HashMap::new();
        let mut still_open: Vec<PendingFile> = Vec::new();
        let mut closed: Vec<CarvedFile> = Vec::new();

        for pending_file in pending {
            let footer_pattern = self.footer_pattern_for(pending_file.kind);
            let footer_matcher = PatternMatcher::new(footer_pattern);
            let search_from = *footer_cursors.get(&pending_file.kind).unwrap_or(&0);

            match footer_matcher.find_in(window.bytes(), search_from) {
                Some(footer_local_idx) => {
                    closed.push(CarvedFile {
                        kind: pending_file.kind,
                        offset_start: pending_file.start_abs,
                        offset_end: window.absolute_offset(footer_local_idx)
                            + footer_pattern.len() as u64,
                    });
                    footer_cursors
                        .insert(pending_file.kind, footer_local_idx + footer_pattern.len());
                }
                None => still_open.push(pending_file),
            }
        }

        (still_open, closed)
    }

    /// Scans the current window for new file headers across all registered
    /// signatures.
    ///
    /// Returns a tuple `(new_pending, found, new_headers)`:
    /// - `new_pending`: headers whose footer was not found in this window yet.
    /// - `found`: files whose header and footer were both found in this window.
    /// - `new_headers`: absolute offsets of all headers discovered in this window,
    ///   to be merged into the caller's deduplication set.
    fn find_new_headers(
        &self,
        window: &WindowSlice,
        known_headers: &HashSet<u64>,
    ) -> (Vec<PendingFile>, Vec<CarvedFile>, Vec<u64>) {
        let mut new_pending: Vec<PendingFile> = Vec::new();
        let mut found: Vec<CarvedFile> = Vec::new();
        let mut new_headers: Vec<u64> = Vec::new();

        for signature in &self.signatures {
            if signature.header_pattern.is_empty() {
                continue;
            }

            let header_matcher = PatternMatcher::new(signature.header_pattern);
            let footer_matcher = PatternMatcher::new(signature.footer_pattern);

            for header_local_idx in header_matcher.find_all_in(window.bytes()) {
                let header_abs = window.absolute_offset(header_local_idx);

                if known_headers.contains(&header_abs) {
                    continue;
                }
                new_headers.push(header_abs);

                let footer_search_from = header_local_idx + signature.header_pattern.len();

                match footer_matcher.find_in(window.bytes(), footer_search_from) {
                    Some(footer_local_idx) => found.push(CarvedFile {
                        kind: signature.kind,
                        offset_start: header_abs,
                        offset_end: window.absolute_offset(footer_local_idx)
                            + signature.footer_pattern.len() as u64,
                    }),
                    None => new_pending.push(PendingFile {
                        kind: signature.kind,
                        start_abs: header_abs,
                    }),
                }
            }
        }

        (new_pending, found, new_headers)
    }

    /// Computes the minimum overlap needed to ensure no pattern spans two
    /// windows without being detected: `max_pattern_length - 1`.
    fn compute_overlap_size(&self) -> usize {
        self.signatures
            .iter()
            .flat_map(|sig| [sig.header_pattern.len(), sig.footer_pattern.len()])
            .max()
            .unwrap_or(0)
            .saturating_sub(1)
    }

    /// Returns the footer pattern for `kind` from the registered signatures,
    /// or an empty slice if the kind is not registered.
    fn footer_pattern_for(&self, kind: FileKind) -> &[u8] {
        self.signatures
            .iter()
            .find(|sig| sig.kind == kind)
            .map(|sig| sig.footer_pattern)
            .unwrap_or(&[])
    }
}
