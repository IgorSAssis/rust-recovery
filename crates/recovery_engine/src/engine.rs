use std::fs;
use std::path::PathBuf;

use file_carver::signature::Signature;
use tracing::{debug, info, instrument};

use crate::error::EngineError;
use crate::strategies::{FileCarverStrategy, RecoveryStrategy};
use crate::types::{ExtractedFile, FileInfo, ReadSeek};

/// Orchestrates scanning and extraction of carved files from any byte source.
///
/// ```ignore
/// let engine = RecoveryEngine::for_carver(SUPPORTED_SIGNATURES.iter().collect(), 4096)
///     .with_output_dir("/tmp/out");
///
/// let extracted = engine.recover(&mut source)?;
/// let paths     = engine.save_all(&extracted)?;
/// ```
pub struct RecoveryEngine {
    output_dir: Option<PathBuf>,
    strategy: Box<dyn RecoveryStrategy>,
}

impl RecoveryEngine {
    /// Creates an engine that uses signature-based file carving.
    pub fn for_carver(signatures: Vec<&'static Signature>, chunk_size: usize) -> Self {
        Self::for_strategy(Box::new(
            FileCarverStrategy::new()
                .with_signatures(signatures)
                .with_chunk_size(chunk_size),
        ))
    }

    /// Creates an engine using a custom recovery strategy.
    pub fn for_strategy(strategy: Box<dyn RecoveryStrategy>) -> Self {
        Self {
            output_dir: None,
            strategy,
        }
    }

    /// Sets (or replaces) the output directory used by [`save_all`].
    pub fn with_output_dir(mut self, output_dir: impl Into<PathBuf>) -> Self {
        self.output_dir = Some(output_dir.into());
        self
    }

    /// Recovers files from `source` using the configured strategy.
    ///
    /// # Errors
    ///
    /// - [`EngineError::NoSignaturesConfigured`] if using the carver strategy
    ///   without signatures.
    /// - [`EngineError::Carver`] or [`EngineError::Filesystem`] depending on
    ///   the active strategy.
    pub fn recover<R: ReadSeek>(&self, source: &mut R) -> Result<Vec<ExtractedFile>, EngineError> {
        self.strategy.recover(source)
    }

    /// Lists recoverable files from `source` without loading their byte content.
    ///
    /// # Errors
    ///
    /// - [`EngineError::NoSignaturesConfigured`] if using the carver strategy
    ///   without signatures.
    /// - [`EngineError::Carver`] on any scanning error.
    #[instrument(name = "engine.scan", skip(self, source))]
    pub fn scan<R: ReadSeek>(&self, source: &mut R) -> Result<Vec<FileInfo>, EngineError> {
        info!("starting scan");
        let file_infos = self.strategy.scan_only(source)?;
        info!(files_found = file_infos.len(), "scan finished");
        Ok(file_infos)
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
