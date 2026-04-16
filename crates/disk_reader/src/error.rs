use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiskError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid offset: tried to seek to {offset} but disk size is {disk_size} bytes")]
    InvalidOffset { offset: u64, disk_size: u64 },

    #[error("Read returned 0 bytes at offset {offset}")]
    UnexpectedEof { offset: u64 },
}
