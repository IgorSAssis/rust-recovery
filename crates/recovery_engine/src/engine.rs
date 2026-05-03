use std::fs;
use std::io::{Read, Seek};
use std::path::PathBuf;

use file_carver::carved_file::CarvedFile;
use file_carver::constants::DEFAULT_CHUNK_SIZE;
use file_carver::extractor::Extractor;
use file_carver::scanner::Scanner;
use file_carver::signature::{FileKind, Signature};
use tracing::{debug, info, instrument};

use crate::error::EngineError;

/// An in-memory representation of a file extracted from a disk source.
///
/// Produced by [`RecoveryEngine::extract_all`] and consumed by
/// [`RecoveryEngine::save_all`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedFile {
    /// Filename to use when saving, e.g. `recovered_0.jpg`.
    pub filename: String,
    pub kind: FileKind,
    pub bytes: Vec<u8>,
}

/// Orchestrates scanning and extraction of carved files from any byte source.
///
/// [`scan`] and [`extract_all`] are generic over `R: Read + Seek`, accepting
/// both `File` and `Cursor<Vec<u8>>` — making them straightforward to
/// unit-test without a real disk or filesystem.
///
/// # Typical workflow
///
/// ```ignore
/// let mut source = File::open("/dev/sdb")?;
/// let engine = RecoveryEngine::new("/tmp/recovered");
///
/// let carved   = engine.scan(&mut source)?;
/// let extracted = engine.extract_all(&mut source, &carved)?;
/// let paths    = engine.save_all(&extracted)?;
/// ```
pub struct RecoveryEngine {
    output_dir: Option<PathBuf>,
    chunk_size: usize,
    signatures: Vec<&'static Signature>,
}

impl Default for RecoveryEngine {
    fn default() -> Self {
        Self {
            output_dir: None,
            chunk_size: DEFAULT_CHUNK_SIZE,
            signatures: Vec::new(),
        }
    }
}

impl RecoveryEngine {
    /// Creates a new engine with no signatures configured.
    ///
    /// Call [`with_signatures`] to specify which file types to detect before
    /// calling [`scan`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets (or replaces) the output directory used by [`save_all`].
    pub fn with_output_dir(mut self, output_dir: impl Into<PathBuf>) -> Self {
        self.output_dir = Some(output_dir.into());
        self
    }

    /// Overrides the number of bytes read per iteration for both scanning and
    /// extraction. Useful for tuning memory usage on large devices.
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Replaces the current signature set with the provided list.
    ///
    /// Pass a subset of [`SUPPORTED_SIGNATURES`] to restrict which file types
    /// are detected during [`scan`].
    pub fn with_signatures(mut self, signatures: Vec<&'static Signature>) -> Self {
        self.signatures = signatures;
        self
    }

    /// Scans `source` from the beginning and returns all detected files,
    /// sorted by their starting offset.
    ///
    /// # Errors
    ///
    /// - [`EngineError::NoSignaturesConfigured`] if the signature list is empty.
    /// - [`EngineError::Carver`] on any scanning error.
    #[instrument(name = "engine.scan", skip(self, source), fields(signatures = self.signatures.len(), chunk_size = self.chunk_size))]
    pub fn scan<R>(&self, source: &mut R) -> Result<Vec<CarvedFile>, EngineError>
    where
        R: Read + Seek,
    {
        if self.signatures.is_empty() {
            return Err(EngineError::NoSignaturesConfigured);
        }

        info!("starting scan");
        let scanner = self.signatures.iter().fold(
            Scanner::new().with_chunk_size(self.chunk_size),
            |scanner, signature| scanner.add_signature(signature),
        );

        let carved_files = scanner.scan(source)?;
        info!(files_found = carved_files.len(), "scan finished");
        Ok(carved_files)
    }

    /// Reads the raw bytes of each file in `carved` from `source` and returns
    /// them as in-memory [`ExtractedFile`] values.
    ///
    /// No filesystem access is performed; the result can be inspected in tests
    /// or passed to [`save_all`] to persist to disk.
    ///
    /// # Errors
    ///
    /// - [`EngineError::Carver`] on any extraction error.
    #[instrument(name = "engine.extract", skip(self, source, carved), fields(files = carved.len()))]
    pub fn extract_all<R>(
        &self,
        source: &mut R,
        carved: &[CarvedFile],
    ) -> Result<Vec<ExtractedFile>, EngineError>
    where
        R: Read + Seek,
    {
        info!("starting extraction");
        let extractor = Extractor::new().with_chunk_size(self.chunk_size);
        let mut extracted_files: Vec<ExtractedFile> = Vec::new();

        for (index, carved_file) in carved.iter().enumerate() {
            let filename = format!("recovered_{}.{}", index, carved_file.kind.extension());
            let mut bytes: Vec<u8> = Vec::new();

            extractor.extract(source, carved_file, &mut bytes)?;

            debug!(
                filename,
                kind = %carved_file.kind,
                bytes = bytes.len(),
                "file extracted"
            );
            extracted_files.push(ExtractedFile {
                filename,
                kind: carved_file.kind,
                bytes,
            });
        }

        info!(files_extracted = extracted_files.len(), "extraction complete");
        Ok(extracted_files)
    }

    /// Writes each [`ExtractedFile`] to `output_dir`, creating it if needed.
    ///
    /// Returns the path of every file written, in the same order as `extracted`.
    ///
    /// # Errors
    ///
    /// - [`EngineError::InvalidOutputDir`] if the output directory cannot be
    ///   created.
    /// - [`EngineError::Io`] if a file cannot be written.
    #[instrument(name = "engine.save", skip(self, extracted), fields(files = extracted.len()))]
    pub fn save_all(&self, extracted: &[ExtractedFile]) -> Result<Vec<PathBuf>, EngineError> {
        let output_dir = self.output_dir.as_ref().ok_or(EngineError::NoOutputDir)?;
        self.ensure_output_dir(output_dir)?;
        info!(output_dir = %output_dir.display(), "saving files");

        let mut saved_paths: Vec<PathBuf> = Vec::new();

        for extracted_file in extracted {
            let output_path = output_dir.join(&extracted_file.filename);
            fs::write(&output_path, &extracted_file.bytes)?;
            debug!(path = %output_path.display(), "file saved");
            saved_paths.push(output_path);
        }

        info!(files_saved = saved_paths.len(), "all files saved");
        Ok(saved_paths)
    }

    fn ensure_output_dir(&self, output_dir: &PathBuf) -> Result<(), EngineError> {
        fs::create_dir_all(output_dir).map_err(|io_error| EngineError::InvalidOutputDir {
            path: output_dir.display().to_string(),
            reason: io_error.to_string(),
        })
    }
}
