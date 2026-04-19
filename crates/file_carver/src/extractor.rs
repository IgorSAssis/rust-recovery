use std::io::{self, Read, Seek, SeekFrom, Write};

use crate::carved_file::CarvedFile;
use crate::constants::DEFAULT_CHUNK_SIZE;
use crate::error::CarverError;

/// Extracts the raw bytes of a [`CarvedFile`] from a byte source and writes
/// them to a destination, reading in fixed-size chunks to avoid loading the
/// entire file into memory.
///
/// # Usage
///
/// ```ignore
/// let mut output = Vec::new();
/// let bytes_written = Extractor::new()
///     .with_chunk_size(4096)
///     .extract(&mut source, &carved_file, &mut output)?;
/// ```
pub struct Extractor {
    chunk_size: usize,
}

impl Default for Extractor {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }
}

impl Extractor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the number of bytes read from the source per iteration.
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Extracts the bytes of `carved` from `source` and writes them to `dest`.
    ///
    /// Seeks `source` to `carved.offset_start`, then copies exactly
    /// `carved.size()` bytes to `dest`.
    ///
    /// Returns the total number of bytes written.
    ///
    /// # Errors
    ///
    /// - [`CarverError::InvalidRange`] if `offset_end <= offset_start`.
    /// - [`CarverError::ExtractionFailed`] on any I/O error, including when the
    ///   source ends before `offset_end` is reached.
    pub fn extract<R, W>(
        &self,
        source: &mut R,
        carved: &CarvedFile,
        dest: &mut W,
    ) -> Result<u64, CarverError>
    where
        R: Read + Seek,
        W: Write,
    {
        Self::validate_range(carved)?;

        source.seek(SeekFrom::Start(carved.offset_start))?;

        let mut remaining: u64 = carved.size();
        let mut total_written: u64 = 0;
        let mut buffer = vec![0u8; self.effective_chunk_size()];

        while remaining > 0 {
            let to_read = self.bytes_to_read(remaining);
            let bytes_read = source.read(&mut buffer[..to_read])?;

            if bytes_read == 0 {
                return Err(Self::unexpected_eof_error(remaining, carved.offset_start));
            }

            dest.write_all(&buffer[..bytes_read])?;
            total_written += bytes_read as u64;
            remaining -= bytes_read as u64;
        }

        Ok(total_written)
    }

    // ── private helpers ───────────────────────────────────────────────────────

    /// Returns the chunk size to use, falling back to [`DEFAULT_CHUNK_SIZE`]
    /// when the configured value is zero.
    fn effective_chunk_size(&self) -> usize {
        if self.chunk_size == 0 {
            DEFAULT_CHUNK_SIZE
        } else {
            self.chunk_size
        }
    }

    /// Returns the number of bytes to attempt to read on the next iteration:
    /// the smaller of the effective chunk size and the bytes still remaining.
    fn bytes_to_read(&self, remaining: u64) -> usize {
        (self.effective_chunk_size() as u64).min(remaining) as usize
    }

    /// Validates that `carved` has a non-empty, well-ordered byte range.
    fn validate_range(carved: &CarvedFile) -> Result<(), CarverError> {
        if carved.offset_end <= carved.offset_start {
            return Err(CarverError::InvalidRange {
                start: carved.offset_start,
                end: carved.offset_end,
            });
        }
        Ok(())
    }

    /// Constructs the error returned when the source stream ends before all
    /// expected bytes have been read.
    fn unexpected_eof_error(remaining: u64, offset_start: u64) -> CarverError {
        CarverError::ExtractionFailed(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!(
                "source ended with {} bytes still remaining for file at offset {}",
                remaining, offset_start
            ),
        ))
    }
}
