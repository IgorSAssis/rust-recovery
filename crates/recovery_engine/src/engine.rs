use std::fs;
use std::io::{Read, Seek};
use std::path::PathBuf;

use file_carver::carved_file::CarvedFile;
use file_carver::constants::DEFAULT_CHUNK_SIZE;
use file_carver::extractor::Extractor;
use file_carver::scanner::Scanner;
use file_carver::signature::{FileKind, Signature, SUPPORTED_SIGNATURES};

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
    output_dir: PathBuf,
    chunk_size: usize,
    signatures: Vec<&'static Signature>,
}

impl RecoveryEngine {
    /// Creates a new engine that saves files to `output_dir`. All supported
    /// signatures (JPEG, PNG) are enabled by default.
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
            chunk_size: DEFAULT_CHUNK_SIZE,
            signatures: SUPPORTED_SIGNATURES.iter().collect(),
        }
    }

    /// Overrides the number of bytes read per iteration for both scanning and
    /// extraction. Useful for tuning memory usage on large devices.
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Adds an extra signature to detect during scanning. The custom signature
    /// is appended to the default set.
    pub fn add_signature(mut self, signature: &'static Signature) -> Self {
        self.signatures.push(signature);
        self
    }

    /// Scans `source` from the beginning and returns all detected files,
    /// sorted by their starting offset.
    ///
    /// # Errors
    ///
    /// - [`EngineError::NoSignaturesConfigured`] if the signature list is empty.
    /// - [`EngineError::Carver`] on any scanning error.
    pub fn scan<R>(&self, source: &mut R) -> Result<Vec<CarvedFile>, EngineError>
    where
        R: Read + Seek,
    {
        if self.signatures.is_empty() {
            return Err(EngineError::NoSignaturesConfigured);
        }

        let scanner = self.signatures.iter().fold(
            Scanner::new().with_chunk_size(self.chunk_size),
            |scanner, signature| scanner.add_signature(signature),
        );

        let carved_files = scanner.scan(source)?;
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
    pub fn extract_all<R>(
        &self,
        source: &mut R,
        carved: &[CarvedFile],
    ) -> Result<Vec<ExtractedFile>, EngineError>
    where
        R: Read + Seek,
    {
        let extractor = Extractor::new().with_chunk_size(self.chunk_size);
        let mut extracted_files: Vec<ExtractedFile> = Vec::new();

        for (index, carved_file) in carved.iter().enumerate() {
            let filename = format!("recovered_{}.{}", index, carved_file.kind.extension());
            let mut bytes: Vec<u8> = Vec::new();

            extractor.extract(source, carved_file, &mut bytes)?;

            extracted_files.push(ExtractedFile {
                filename,
                kind: carved_file.kind,
                bytes,
            });
        }

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
    pub fn save_all(&self, extracted: &[ExtractedFile]) -> Result<Vec<PathBuf>, EngineError> {
        self.ensure_output_dir()?;

        let mut saved_paths: Vec<PathBuf> = Vec::new();

        for extracted_file in extracted {
            let output_path = self.output_dir.join(&extracted_file.filename);
            fs::write(&output_path, &extracted_file.bytes)?;
            saved_paths.push(output_path);
        }

        Ok(saved_paths)
    }

    fn ensure_output_dir(&self) -> Result<(), EngineError> {
        fs::create_dir_all(&self.output_dir).map_err(|io_error| EngineError::InvalidOutputDir {
            path: self.output_dir.display().to_string(),
            reason: io_error.to_string(),
        })
    }
}

