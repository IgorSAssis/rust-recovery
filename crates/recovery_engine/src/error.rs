use thiserror::Error;

use disk_reader::error::DiskError;
use file_carver::error::CarverError;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Disk error: {0}")]
    Disk(#[from] DiskError),

    #[error("Carver error: {0}")]
    Carver(#[from] CarverError),

    #[error("Unknown error")]
    Unknown,
}
