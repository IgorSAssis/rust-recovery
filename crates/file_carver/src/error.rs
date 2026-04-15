use thiserror::Error;

#[derive(Debug, Error)]
pub enum CarverError {
    #[error("Signature not found")]
    SignatureNotFound,

    #[error("Footer not found")]
    FooterNotFound,

    #[error("Invalid file range")]
    InvalidRange,

    #[error("Extraction failed")]
    ExtractionFailed,
}
