use thiserror::Error;

use crate::signature::FileKind;

#[derive(Debug, Error)]
pub enum CarverError {
    #[error("No header found for signature '{kind}'")]
    SignatureNotFound { kind: FileKind },

    #[error("Footer not found for '{kind}' (header at offset {header_offset})")]
    FooterNotFound { kind: FileKind, header_offset: usize },

    #[error("Invalid file range: start={start}, end={end} (end must be greater than start)")]
    InvalidRange { start: usize, end: usize },

    #[error("Failed to extract file: {0}")]
    ExtractionFailed(#[from] std::io::Error),
}
