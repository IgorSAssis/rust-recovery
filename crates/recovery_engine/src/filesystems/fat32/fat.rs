use std::io::{Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::error::EngineError;

use super::boot_sector::Fat32BootSector;

/// Maximum cluster chain length before aborting — guards against infinite loops
/// in corrupted FATs where entries form a cycle.
const MAX_CHAIN_LENGTH: usize = 1_000_000;

/// Minimum value of an End-of-Chain (EOC) FAT32 marker (after masking).
pub const FAT_EOC_MIN: u32 = 0x0FFF_FFF8;

/// Mask applied to every raw FAT32 entry: only the lower 28 bits are used.
const FAT_MASK: u32 = 0x0FFF_FFFF;

/// Reads the FAT32 table entry for `cluster` from `source`.
///
/// The File Allocation Table is a flat array of 4-byte little-endian values
/// located at `boot.fat_start_offset()`.  The entry for cluster `N` sits at:
///
/// ```text
/// fat_start + N * 4
/// ```
///
/// The raw value is masked with `0x0FFFFFFF` because the upper 4 bits are
/// reserved and must be ignored.
///
/// ## Interpretation of the returned value
///
/// | Value range          | Meaning                         |
/// |----------------------|---------------------------------|
/// | `0x00000000`         | Free cluster                    |
/// | `0x00000001`         | Reserved                        |
/// | `0x00000002..MAX`    | Next cluster in the chain       |
/// | `0x0FFFFFF7`         | Bad cluster (do not use)        |
/// | `0x0FFFFFF8..=0x0FFFFFFF` | End of chain                |
pub fn read_fat_entry<R: Read + Seek + ?Sized>(
    source: &mut R,
    boot: &Fat32BootSector,
    cluster: u32,
) -> Result<u32, EngineError> {
    let offset = boot.fat_start_offset() + cluster as u64 * 4;
    source.seek(SeekFrom::Start(offset))?;
    let raw = source.read_u32::<LittleEndian>()?;
    Ok(raw & FAT_MASK)
}

/// Walks the FAT starting at `start_cluster` and collects every cluster number
/// in the chain (inclusive of `start_cluster`).
///
/// Terminates when it encounters an End-of-Chain marker (`>= 0x0FFFFFF8`), a
/// free/reserved cluster (`< 2`), or after [`MAX_CHAIN_LENGTH`] entries (to
/// guard against loops in corrupted FATs).
pub fn collect_cluster_chain<R: Read + Seek + ?Sized>(
    source: &mut R,
    boot: &Fat32BootSector,
    start_cluster: u32,
) -> Result<Vec<u32>, EngineError> {
    let mut chain = Vec::new();
    let mut current = start_cluster;

    loop {
        if chain.len() >= MAX_CHAIN_LENGTH {
            return Err(EngineError::InvalidFilesystem {
                reason: format!(
                    "cluster chain exceeded {MAX_CHAIN_LENGTH} entries — possible FAT loop at cluster {current}"
                ),
            });
        }

        chain.push(current);

        let next = read_fat_entry(source, boot, current)?;

        if next >= FAT_EOC_MIN {
            break; // end of chain
        }

        if next < 2 {
            break; // free or reserved cluster — chain is broken
        }

        current = next;
    }

    Ok(chain)
}

#[cfg(test)]
#[path = "fat_tests.rs"]
mod tests;
