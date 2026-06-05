use std::io::{self, Cursor, Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::error::EngineError;

/// The FAT32 Boot Sector (also called the BPB — BIOS Parameter Block).
///
/// Located at sector 0, byte 0 of the volume.  The first 512 bytes encode
/// all the parameters needed to navigate the rest of the filesystem.
///
/// # Field layout in the 512-byte sector
///
/// ```text
/// Offset  Size  Field
/// ──────  ────  ─────────────────────────────────
///  0       3    Jump code (ignored)
///  3       8    OEM name string (ignored)
/// 11       2    bytes_per_sector
/// 13       1    sectors_per_cluster
/// 14       2    reserved_sectors
/// 16       1    num_fats
/// 17       2    root_entry_count   (must be 0 for FAT32)
/// 19       2    total_sectors_16   (must be 0 for FAT32)
/// 21       1    media_descriptor   (ignored)
/// 22       2    fat_size_16        (must be 0 for FAT32)
/// 24       2    sectors_per_track  (ignored)
/// 26       2    num_heads          (ignored)
/// 28       4    hidden_sectors     (ignored)
/// 32       4    total_sectors_32
/// 36       4    fat_size_32
/// 40       2    ext_flags          (ignored)
/// 42       2    fs_version         (ignored)
/// 44       4    root_cluster
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fat32BootSector {
    /// Bytes in one logical sector (typically 512).
    pub bytes_per_sector: u16,
    /// Logical sectors per allocation cluster (must be a power of two).
    pub sectors_per_cluster: u8,
    /// Sectors before the first FAT (includes this boot sector).
    pub reserved_sectors: u16,
    /// Number of FAT copies on disk (typically 2).
    pub num_fats: u8,
    /// Total logical sectors on the volume.
    pub total_sectors_32: u32,
    /// Sectors occupied by one FAT copy.
    pub fat_size_32: u32,
    /// First cluster of the root directory (typically 2).
    pub root_cluster: u32,
}

impl Fat32BootSector {
    /// Reads and parses the FAT32 Boot Sector from `source`.
    ///
    /// Seeks to byte 0, reads exactly 512 bytes and extracts the relevant
    /// fields.  Returns [`EngineError::InvalidFilesystem`] if the bytes do not
    /// look like a valid FAT32 volume (e.g. `bytes_per_sector` is zero, or
    /// FAT32-specific fields are absent).
    pub fn parse<R: Read + Seek + ?Sized>(source: &mut R) -> Result<Self, EngineError> {
        source.seek(SeekFrom::Start(0))?;

        let mut raw = [0u8; 512];
        source
            .read_exact(&mut raw)
            .map_err(|e| EngineError::InvalidFilesystem {
                reason: format!("failed to read boot sector: {e}"),
            })?;

        Self::parse_bytes(&raw)
    }

    /// Parses the boot sector from a raw 512-byte buffer.
    ///
    /// Separated from [`parse`] so that tests can pass hand-crafted byte
    /// arrays without requiring a real filesystem image or file handle.
    pub fn parse_bytes(raw: &[u8; 512]) -> Result<Self, EngineError> {
        let mut cur = Cursor::new(raw.as_slice());

        // Skip: jump code (3) + OEM name (8) = 11 bytes.
        cur.seek(SeekFrom::Start(11)).map_err(io_err)?;

        let bytes_per_sector = cur.read_u16::<LittleEndian>().map_err(io_err)?; // offset 11
        let sectors_per_cluster = cur.read_u8().map_err(io_err)?; // offset 13
        let reserved_sectors = cur.read_u16::<LittleEndian>().map_err(io_err)?; // offset 14
        let num_fats = cur.read_u8().map_err(io_err)?; // offset 16

        // Jump to offset 32, skipping fields not needed for navigation:
        // root_entry_count(2) + total_sectors_16(2) + media(1) + fat_size_16(2)
        // + sectors_per_track(2) + num_heads(2) + hidden_sectors(4) = 15 bytes.
        cur.seek(SeekFrom::Start(32)).map_err(io_err)?;

        let total_sectors_32 = cur.read_u32::<LittleEndian>().map_err(io_err)?; // offset 32
        let fat_size_32 = cur.read_u32::<LittleEndian>().map_err(io_err)?; // offset 36

        // Jump to offset 44, skipping ext_flags(2) + fs_version(2).
        cur.seek(SeekFrom::Start(44)).map_err(io_err)?;

        let root_cluster = cur.read_u32::<LittleEndian>().map_err(io_err)?; // offset 44

        let boot = Self {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sectors,
            num_fats,
            total_sectors_32,
            fat_size_32,
            root_cluster,
        };

        boot.validate()?;
        Ok(boot)
    }

    /// Returns the size of one allocation cluster in bytes.
    pub fn cluster_size(&self) -> u64 {
        self.bytes_per_sector as u64 * self.sectors_per_cluster as u64
    }

    /// Returns the byte offset of the first FAT on disk.
    pub fn fat_start_offset(&self) -> u64 {
        self.reserved_sectors as u64 * self.bytes_per_sector as u64
    }

    /// Returns the byte offset of the data area (first usable cluster, #2).
    ///
    /// This is the reference point for all cluster address calculations:
    /// `cluster_offset(n) = data_start + (n - 2) * cluster_size`
    pub fn data_start_offset(&self) -> u64 {
        let fat_bytes =
            self.num_fats as u64 * self.fat_size_32 as u64 * self.bytes_per_sector as u64;
        self.fat_start_offset() + fat_bytes
    }

    /// Returns the absolute byte offset of `cluster` within the image.
    ///
    /// FAT32 cluster numbering starts at 2 — clusters 0 and 1 are reserved.
    /// Returns [`None`] for values less than 2.
    pub fn cluster_offset(&self, cluster: u32) -> Option<u64> {
        if cluster < 2 {
            return None;
        }
        Some(self.data_start_offset() + (cluster as u64 - 2) * self.cluster_size())
    }

    fn validate(&self) -> Result<(), EngineError> {
        if self.bytes_per_sector == 0 {
            return Err(EngineError::InvalidFilesystem {
                reason: "bytes_per_sector is 0".into(),
            });
        }
        if !self.sectors_per_cluster.is_power_of_two() {
            return Err(EngineError::InvalidFilesystem {
                reason: format!(
                    "sectors_per_cluster ({}) must be a power of two",
                    self.sectors_per_cluster
                ),
            });
        }
        if self.num_fats == 0 {
            return Err(EngineError::InvalidFilesystem {
                reason: "num_fats is 0".into(),
            });
        }
        if self.fat_size_32 == 0 {
            return Err(EngineError::InvalidFilesystem {
                reason: "fat_size_32 is 0 — not a FAT32 volume".into(),
            });
        }
        Ok(())
    }
}

fn io_err(e: io::Error) -> EngineError {
    EngineError::InvalidFilesystem {
        reason: e.to_string(),
    }
}

#[cfg(test)]
#[path = "boot_sector_tests.rs"]
mod tests;
