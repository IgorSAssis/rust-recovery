use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::error::DiskError;

#[derive(Debug)]
pub struct DiskReader {
    file: File,
}

impl DiskReader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DiskError> {
        let file = File::open(path)?;
        Ok(Self { file })
    }

    pub fn file_size(&self) -> Result<u64, DiskError> {
        Ok(self.file.metadata()?.len())
    }

    pub fn read_chunk(&mut self, buffer: &mut [u8]) -> Result<usize, DiskError> {
        let bytes_read = self.file.read(buffer)?;
        Ok(bytes_read)
    }

    pub fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize, DiskError> {
        self.file.seek(SeekFrom::Start(offset))?;
        let bytes_read = self.read_chunk(buffer)?;
        Ok(bytes_read)
    }
}
