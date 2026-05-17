use std::collections::{HashMap, HashSet};
use std::io::{self, Read, Seek, SeekFrom};

use tracing::{debug, info, instrument};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use crate::carved_file::CarvedFile;
use crate::constants::DEFAULT_CHUNK_SIZE;
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
///     .with_chunk_size(DEFAULT_CHUNK_SIZE)
///     .scan(&mut source)?;
/// ```
pub struct Scanner {
    chunk_size: usize,
    signatures: Vec<&'static Signature>,
}

impl Default for Scanner {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
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
    #[instrument(name = "scanner.scan", skip(self, source), fields(signatures = self.signatures.len(), chunk_size = self.chunk_size))]
    pub fn scan<R: Read + Seek + ?Sized>(&self, source: &mut R) -> io::Result<Vec<CarvedFile>> {
        if self.signatures.is_empty() || self.chunk_size == 0 {
            return Ok(Vec::new());
        }

        source.seek(SeekFrom::Start(0))?;

        let overlap_size = self.compute_overlap_size();
        let mut scan_window = ScanWindow::new(overlap_size, self.chunk_size);

        let mut carved: Vec<CarvedFile> = Vec::new();
        let mut pending: Vec<PendingFile> = Vec::new();
        let mut known_headers: HashSet<u64> = HashSet::new();
        let mut chunk_index: u64 = 0;

        while let Some(window) = scan_window.next(source)? {
            let (still_open, closed) = self.close_pending(pending, &window);
            let (new_pending, found, new_headers) = self.find_new_headers(&window, &known_headers);

            let headers_found = new_headers.len();
            let footers_closed = closed.len() + found.len();

            carved.extend(closed);
            carved.extend(found);
            pending = still_open;
            pending.extend(new_pending);
            known_headers.extend(new_headers);

            debug!(
                chunk = chunk_index,
                base_offset = window.absolute_offset(0),
                bytes = window.bytes().len(),
                headers_found,
                footers_closed,
                pending = pending.len(),
                "chunk processed"
            );
            chunk_index += 1;
        }

        info!(files_found = carved.len(), "scan complete");
        carved.sort_by_key(|carved_file| carved_file.offset_start);
        Ok(carved)
    }

    // ── private helpers ───────────────────────────────────────────────────────

    /// Attempts to close each pending file by searching for its footer in the
    /// current window. Returns `(still_open, closed)`.
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
                    let offset_end =
                        window.absolute_offset(footer_local_idx) + footer_pattern.len() as u64;
                    debug!(
                        kind = %pending_file.kind,
                        offset_start = pending_file.start_abs,
                        offset_end,
                        "footer found, file closed"
                    );
                    closed.push(CarvedFile {
                        kind: pending_file.kind,
                        offset_start: pending_file.start_abs,
                        offset_end,
                    });
                    footer_cursors
                        .insert(pending_file.kind, footer_local_idx + footer_pattern.len());
                }
                None => still_open.push(pending_file),
            }
        }

        (still_open, closed)
    }

    /// Scans the current window for new file headers across all registered signatures.
    ///
    /// Returns `(new_pending, found, new_headers)`. When compiled with the `parallel`
    /// feature, each signature is searched concurrently via `rayon`.
    fn find_new_headers(
        &self,
        window: &WindowSlice,
        known_headers: &HashSet<u64>,
    ) -> (Vec<PendingFile>, Vec<CarvedFile>, Vec<u64>) {
        #[cfg(feature = "parallel")]
        let iter = self.signatures.par_iter();
        #[cfg(not(feature = "parallel"))]
        let iter = self.signatures.iter();

        let per_signature: Vec<(Vec<PendingFile>, Vec<CarvedFile>, Vec<u64>)> = iter
            .map(|signature| {
                Self::search_signature(signature, window, known_headers)
            })
            .collect();

        let mut new_pending: Vec<PendingFile> = Vec::new();
        let mut found: Vec<CarvedFile> = Vec::new();
        let mut new_headers: Vec<u64> = Vec::new();

        for (sig_pending, sig_found, sig_headers) in per_signature {
            new_pending.extend(sig_pending);
            found.extend(sig_found);
            new_headers.extend(sig_headers);
        }

        (new_pending, found, new_headers)
    }

    /// Searches a single signature's header and footer within `window`.
    fn search_signature(
        signature: &Signature,
        window: &WindowSlice,
        known_headers: &HashSet<u64>,
    ) -> (Vec<PendingFile>, Vec<CarvedFile>, Vec<u64>) {
        let mut sig_pending: Vec<PendingFile> = Vec::new();
        let mut sig_found: Vec<CarvedFile> = Vec::new();
        let mut sig_headers: Vec<u64> = Vec::new();

        if signature.header_pattern.is_empty() {
            return (sig_pending, sig_found, sig_headers);
        }

        let header_matcher = PatternMatcher::new(signature.header_pattern);
        let footer_matcher = PatternMatcher::new(signature.footer_pattern);

        for header_local_idx in header_matcher.find_all_in(window.bytes()) {
            let header_abs = window.absolute_offset(header_local_idx);

            if known_headers.contains(&header_abs) {
                continue;
            }
            sig_headers.push(header_abs);

            let footer_search_from = header_local_idx + signature.header_pattern.len();

            match footer_matcher.find_in(window.bytes(), footer_search_from) {
                Some(footer_local_idx) => {
                    let offset_end = window.absolute_offset(footer_local_idx)
                        + signature.footer_pattern.len() as u64;
                    debug!(
                        kind = %signature.kind,
                        offset_start = header_abs,
                        offset_end,
                        "file carved (header + footer in same window)"
                    );
                    sig_found.push(CarvedFile {
                        kind: signature.kind,
                        offset_start: header_abs,
                        offset_end,
                    });
                }
                None => {
                    debug!(kind = %signature.kind, offset = header_abs, "header found, awaiting footer");
                    sig_pending.push(PendingFile {
                        kind: signature.kind,
                        start_abs: header_abs,
                    });
                }
            }
        }

        (sig_pending, sig_found, sig_headers)
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
