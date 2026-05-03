use file_carver::constants::DEFAULT_CHUNK_SIZE;
use file_carver::extractor::Extractor;
use file_carver::scanner::Scanner;
use file_carver::signature::Signature;
use tracing::{debug, info, instrument};

use crate::error::EngineError;
use crate::types::{ExtractedFile, ReadSeek};

use super::RecoveryStrategy;

/// A [`RecoveryStrategy`] that scans raw bytes for file signatures and extracts
/// the files it finds — the "file carving" approach.
///
/// This strategy does not require any knowledge of the filesystem; it works on
/// any raw byte source including formatted or partially corrupted devices.
///
/// Build it with the builder API, then pass it to
/// [`RecoveryEngine::with_strategy`]:
///
/// ```ignore
/// let strategy = FileCarverStrategy::new()
///     .with_signatures(SUPPORTED_SIGNATURES.iter().collect())
///     .with_chunk_size(4096);
///
/// let engine = RecoveryEngine::new()
///     .with_output_dir("/tmp/out")
///     .with_strategy(Box::new(strategy));
/// ```
pub struct FileCarverStrategy {
    chunk_size: usize,
    signatures: Vec<&'static Signature>,
}

impl FileCarverStrategy {
    pub fn new() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            signatures: Vec::new(),
        }
    }

    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    pub fn with_signatures(mut self, signatures: Vec<&'static Signature>) -> Self {
        self.signatures = signatures;
        self
    }
}

impl Default for FileCarverStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RecoveryStrategy for FileCarverStrategy {
    #[instrument(
        name = "strategy.carver.recover",
        skip(self, source),
        fields(signatures = self.signatures.len(), chunk_size = self.chunk_size)
    )]
    fn recover(&self, source: &mut dyn ReadSeek) -> Result<Vec<ExtractedFile>, EngineError> {
        if self.signatures.is_empty() {
            return Err(EngineError::NoSignaturesConfigured);
        }

        info!("starting file carver recovery");

        let scanner = self.signatures.iter().fold(
            Scanner::new().with_chunk_size(self.chunk_size),
            |scanner, sig| scanner.add_signature(sig),
        );

        let carved_files = scanner.scan(source)?;
        let extractor = Extractor::new().with_chunk_size(self.chunk_size);
        let mut extracted: Vec<ExtractedFile> = Vec::new();

        for (i, carved) in carved_files.iter().enumerate() {
            let filename = format!("recovered_{}.{}", i, carved.kind.extension());
            let mut bytes: Vec<u8> = Vec::new();
            extractor.extract(source, carved, &mut bytes)?;
            debug!(filename, bytes = bytes.len(), "file extracted");
            extracted.push(ExtractedFile {
                filename,
                kind: carved.kind,
                bytes,
            });
        }

        info!(
            files_recovered = extracted.len(),
            "file carver recovery complete"
        );
        Ok(extracted)
    }
}
