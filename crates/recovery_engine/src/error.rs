use thiserror::Error;

use file_carver::error::CarverError;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Carver error: {0}")]
    Carver(#[from] CarverError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not create output directory '{path}': {reason}")]
    InvalidOutputDir { path: String, reason: String },

    #[error("Scan aborted: no supported signatures configured")]
    NoSignaturesConfigured,
}
