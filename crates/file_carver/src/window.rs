use std::io::{self, Read};

/// The content of a single scan window: the tail of the previous chunk
/// (overlap) followed by the bytes of the current chunk.
///
/// `WindowSlice` is the unit of work consumed by the scanner on each
/// iteration — it represents a contiguous view of the source with a known
/// position in the disk address space.
pub(crate) struct WindowSlice {
    bytes: Vec<u8>,
    /// Absolute disk offset of `bytes[0]`.
    base: u64,
}

impl WindowSlice {
    /// Returns a slice over the window bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Converts a local index within this window into an absolute byte offset
    /// within the source device or image.
    ///
    /// Centralises the `base + local_idx` arithmetic that would otherwise be
    /// repeated at every call site inside the scanner.
    pub fn absolute_offset(&self, local_idx: usize) -> u64 {
        self.base + local_idx as u64
    }
}

/// Reads a source in fixed-size chunks while maintaining an overlap region
/// between consecutive windows.
///
/// The overlap ensures that byte patterns spanning a chunk boundary are
/// always fully visible within a single [`WindowSlice`], so the scanner never
/// misses a signature that straddles two chunks.
///
/// # Overlap size
///
/// The caller must set `overlap_size` to at least `max_pattern_length - 1`,
/// where `max_pattern_length` is the longest header or footer pattern among
/// all signatures being searched.
pub(crate) struct ScanWindow {
    overlap_size: usize,
    chunk_buf: Vec<u8>,
    overlap: Vec<u8>,
    /// Absolute position in the source **before** the most recent read.
    disk_pos: u64,
}

impl ScanWindow {
    /// Creates a new `ScanWindow`.
    ///
    /// - `overlap_size`: number of bytes carried forward from each chunk to
    ///   bridge chunk boundaries.
    /// - `chunk_size`: number of bytes read from the source per iteration.
    pub fn new(overlap_size: usize, chunk_size: usize) -> Self {
        Self {
            overlap_size,
            chunk_buf: vec![0u8; chunk_size],
            overlap: Vec::new(),
            disk_pos: 0,
        }
    }

    /// Reads the next chunk from `source` and returns a [`WindowSlice`]
    /// containing the overlap from the previous call followed by the new bytes.
    ///
    /// Returns `Ok(None)` when the source is exhausted (EOF).
    pub fn next<R: Read>(&mut self, source: &mut R) -> io::Result<Option<WindowSlice>> {
        let bytes_read = source.read(&mut self.chunk_buf)?;

        if bytes_read == 0 {
            return Ok(None);
        }

        let new_bytes = &self.chunk_buf[..bytes_read];

        // `base` is computed with the disk position BEFORE the current read
        // so that window.bytes[0] maps to the correct absolute disk offset.
        let base = self.disk_pos.saturating_sub(self.overlap.len() as u64);

        let mut window_bytes = self.overlap.clone();
        window_bytes.extend_from_slice(new_bytes);

        self.disk_pos += bytes_read as u64;

        // Carry forward the last `overlap_size` bytes of the new chunk so the
        // next window can see patterns that started near the end of this one.
        let tail_start = new_bytes.len().saturating_sub(self.overlap_size);
        self.overlap = new_bytes[tail_start..].to_vec();

        Ok(Some(WindowSlice { bytes: window_bytes, base }))
    }
}
