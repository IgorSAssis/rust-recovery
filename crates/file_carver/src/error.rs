use thiserror::Error;

#[derive(Debug, Error)]
pub enum CarverError {
    #[error("No header found for signature '{signature}'")]
    SignatureNotFound { signature: &'static str },

    #[error(
        "Footer not found for signature '{signature}' (header at offset {header_offset})"
    )]
    FooterNotFound {
        signature: &'static str,
        header_offset: usize,
    },

    #[error("Invalid file range: start={start}, end={end} (end must be greater than start)")]
    InvalidRange { start: usize, end: usize },

    #[error("Failed to extract file: {0}")]
    ExtractionFailed(#[from] std::io::Error),
}
