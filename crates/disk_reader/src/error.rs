use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiskError {
    #[error("Failed to open file: {0}")]
    OpenError(String),

    #[error("Failed to read file")]
    ReadError,

    #[error("Seek failed")]
    SeekError,

    #[error("Invalid offset")]
    InvalidOffset,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
