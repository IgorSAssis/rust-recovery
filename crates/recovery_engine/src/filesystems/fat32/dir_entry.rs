use std::io::{Read, Seek, SeekFrom};

use crate::error::EngineError;

use super::boot_sector::Fat32BootSector;
use super::fat::collect_cluster_chain;

/// Attribute byte value that marks a Long File Name (LFN) sub-entry.
const ATTR_LFN: u8 = 0x0F;

/// Attribute bit that marks a volume label entry.
const ATTR_VOLUME_BIT: u8 = 0x08;

/// First byte of a deleted directory entry.
pub const DELETED_MARKER: u8 = 0xE5;

/// First byte that signals the end of the directory (no more entries follow).
pub const END_OF_DIR: u8 = 0x00;

/// A raw 32-byte FAT32 directory entry.
///
/// FAT32 stores file metadata in fixed-size 32-byte slots inside directory
/// clusters.  Each slot encodes the 8.3 filename, attributes, timestamps,
/// the starting cluster of the file's data, and the file size.
///
/// # Layout
///
/// ```text
/// Offset  Size  Field
/// ──────  ────  ─────────────────────────────────
///  0       8    Short name   (space-padded, 0xE5 = deleted, 0x00 = end)
///  8       3    Extension    (space-padded)
/// 11       1    Attributes   (0x0F = LFN sub-entry)
/// 12–19   …    Timestamps   (ignored for recovery)
/// 20       2    First cluster (high word)
/// 22–25   …    Write time / date (ignored)
/// 26       2    First cluster (low word)
/// 28       4    File size    (bytes)
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fat32RawDirEntry {
    raw: [u8; 32],
}

impl Fat32RawDirEntry {
    /// Wraps a raw 32-byte buffer as a directory entry.
    pub fn from_raw(raw: [u8; 32]) -> Self {
        Self { raw }
    }

    /// Returns `true` if this entry has been deleted (`raw[0] == 0xE5`).
    ///
    /// When a file is deleted in FAT32, the filesystem overwrites only the
    /// first byte of the name with `0xE5`.  All other fields — including
    /// cluster number and file size — remain intact until the space is reused.
    pub fn is_deleted(&self) -> bool {
        self.raw[0] == DELETED_MARKER
    }

    /// Returns `true` if this slot marks the end of the directory.
    ///
    /// A `0x00` first byte means: "no entries exist at or after this slot."
    /// Scanning must stop here.
    pub fn is_end_of_dir(&self) -> bool {
        self.raw[0] == END_OF_DIR
    }

    /// Returns `true` if this is a Long File Name (LFN) sub-entry.
    ///
    /// LFN entries store Unicode filename parts across multiple 32-byte slots.
    /// They are identified by having all four attribute bits set (`0x0F`).
    /// Recovery only needs the 8.3 short name, so LFN entries are skipped.
    pub fn is_long_name_entry(&self) -> bool {
        self.raw[11] == ATTR_LFN
    }

    /// Returns `true` if this entry is the volume label.
    ///
    /// Volume labels have the volume-bit set but are not LFN entries.
    /// They do not represent recoverable files.
    pub fn is_volume_label(&self) -> bool {
        self.raw[11] & ATTR_VOLUME_BIT != 0 && self.raw[11] != ATTR_LFN
    }

    /// Returns the 32-bit first cluster number for this entry.
    ///
    /// FAT32 splits the cluster address across two 16-bit fields:
    /// the **high word** at offset 20 and the **low word** at offset 26.
    /// Combining them: `(high << 16) | low`.
    pub fn first_cluster(&self) -> u32 {
        let high = u16::from_le_bytes([self.raw[20], self.raw[21]]) as u32;
        let low = u16::from_le_bytes([self.raw[26], self.raw[27]]) as u32;
        (high << 16) | low
    }

    /// Returns the recorded file size in bytes (from offset 28).
    ///
    /// Note: for deleted files this value may be stale if the cluster chain
    /// has been partially overwritten by a new file.
    pub fn file_size(&self) -> u32 {
        u32::from_le_bytes(self.raw[28..32].try_into().unwrap())
    }

    /// Reconstructs the 8.3 short name as a human-readable `String`.
    ///
    /// * Trailing spaces are trimmed from both the name and extension parts.
    /// * If the first byte is the deleted marker (`0xE5`), it is replaced with
    ///   `'?'` because the original character is lost.
    /// * If the extension is non-empty the result is `"NAME.EXT"`.
    pub fn short_name(&self) -> String {
        let mut name_bytes = [0u8; 8];
        name_bytes.copy_from_slice(&self.raw[0..8]);
        if name_bytes[0] == DELETED_MARKER {
            name_bytes[0] = b'?';
        }

        let name = std::str::from_utf8(&name_bytes)
            .unwrap_or("????????")
            .trim_end_matches(' ')
            .to_string();

        let ext = std::str::from_utf8(&self.raw[8..11])
            .unwrap_or("???")
            .trim_end_matches(' ')
            .to_string();

        if ext.is_empty() {
            name
        } else {
            format!("{name}.{ext}")
        }
    }
}

/// Metadata about a deleted file found in a FAT32 directory cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedEntry {
    /// Best-effort reconstructed 8.3 filename.
    ///
    /// The first character is always `'?'` for deleted entries because FAT32
    /// overwrites it with the deletion marker `0xE5`.
    pub name: String,

    /// Starting cluster of the file's data region.
    pub first_cluster: u32,

    /// Recorded file size in bytes.
    ///
    /// Treat this as advisory — it reflects the size at deletion time and may
    /// be incorrect if cluster data was subsequently overwritten.
    pub file_size: u32,
}

/// Scans the root directory of a FAT32 volume and returns all deleted entries.
///
/// 1. Reads `boot.root_cluster` and follows the FAT chain to collect every
///    cluster belonging to the root directory.
/// 2. For each cluster, iterates over all 32-byte directory slots.
/// 3. Skips LFN sub-entries and volume labels.
/// 4. Stops at the first end-of-directory marker (`raw[0] == 0x00`).
/// 5. Collects every slot where `raw[0] == 0xE5` (deleted).
pub fn list_deleted_entries<R: Read + Seek + ?Sized>(
    source: &mut R,
    boot: &Fat32BootSector,
) -> Result<Vec<DeletedEntry>, EngineError> {
    let chain = collect_cluster_chain(source, boot, boot.root_cluster)?;
    let entries_per_cluster = (boot.cluster_size() / 32) as usize;
    let mut deleted = Vec::new();

    // `'outer` labels the cluster loop so that `break 'outer` from the inner
    // entry loop can exit both levels when end-of-dir is reached.
    'outer: for cluster in chain {
        let cluster_start =
            boot.cluster_offset(cluster)
                .ok_or_else(|| EngineError::InvalidFilesystem {
                    reason: format!("invalid cluster {cluster} in root directory chain"),
                })?;

        for i in 0..entries_per_cluster {
            let entry_offset = cluster_start + (i as u64 * 32);
            source.seek(SeekFrom::Start(entry_offset))?;

            let mut raw = [0u8; 32];
            source.read_exact(&mut raw)?;
            let entry = Fat32RawDirEntry::from_raw(raw);

            if entry.is_end_of_dir() {
                break 'outer;
            }

            if entry.is_long_name_entry() || entry.is_volume_label() {
                continue;
            }

            if entry.is_deleted() {
                deleted.push(DeletedEntry {
                    name: entry.short_name(),
                    first_cluster: entry.first_cluster(),
                    file_size: entry.file_size(),
                });
            }
        }
    }

    Ok(deleted)
}

#[cfg(test)]
#[path = "dir_entry_tests.rs"]
mod tests;
