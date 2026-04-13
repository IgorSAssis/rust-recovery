use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug)]
pub struct DiskReader {
    file: File,
}

impl DiskReader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        Ok(Self { file })
    }

    pub fn read_chunk(&mut self, buffer: &mut [u8]) -> Result<usize> {
        self.file.read(buffer)
    }

    pub fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        self.file.seek(SeekFrom::Start(offset))?;
        self.read_chunk(buffer)
    }
}
