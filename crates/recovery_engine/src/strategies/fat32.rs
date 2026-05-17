use std::io::{Read, Seek, SeekFrom};

use tracing::{debug, info, instrument, warn};

use crate::error::EngineError;
use crate::filesystems::fat32::{DeletedEntry, Fat32BootSector, list_deleted_entries};
use crate::types::{ExtractedFile, ReadSeek};

use super::RecoveryStrategy;

/// A [`RecoveryStrategy`] that uses the FAT32 filesystem metadata to locate
/// and recover deleted files.
///
/// Unlike [`super::FileCarverStrategy`], this strategy does **not** scan raw
/// bytes for signatures.  Instead, it:
///
/// 1. Parses the FAT32 Boot Sector to understand volume geometry.
/// 2. Walks the root directory cluster chain to find deleted entries
///    (those whose first name byte was overwritten with `0xE5`).
/// 3. Reads each deleted file's data by assuming **contiguous cluster
///    allocation** — FAT entries are zeroed on deletion, but clusters are
///    usually allocated contiguously for recently written files, making this
///    a reliable recovery heuristic.
///
/// ## Limitations
///
/// - Skips entries with no extension (e.g. bare names like `README`).
/// - Assumes cluster data has not been overwritten since deletion.
pub struct Fat32Strategy;

impl Fat32Strategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Fat32Strategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RecoveryStrategy for Fat32Strategy {
    #[instrument(name = "strategy.fat32.recover", skip(self, source))]
    fn recover(&self, source: &mut dyn ReadSeek) -> Result<Vec<ExtractedFile>, EngineError> {
        info!("starting FAT32 recovery");

        let boot = Fat32BootSector::parse(source)?;
        debug!(
            bytes_per_sector = boot.bytes_per_sector,
            sectors_per_cluster = boot.sectors_per_cluster,
            cluster_size = boot.cluster_size(),
            root_cluster = boot.root_cluster,
            "FAT32 boot sector parsed"
        );

        let deleted = list_deleted_entries(source, &boot)?;
        info!(
            deleted_count = deleted.len(),
            "deleted directory entries found"
        );

        let mut extracted = Vec::new();

        for entry in &deleted {
            let extension = match extension_from_entry(entry) {
                Some(ext) => ext,
                None => {
                    warn!(name = entry.name, "no extension — skipping");
                    continue;
                }
            };

            if entry.first_cluster < 2 {
                warn!(
                    name = entry.name,
                    cluster = entry.first_cluster,
                    "invalid first cluster — skipping"
                );
                continue;
            }

            let bytes = read_contiguous(source, &boot, entry.first_cluster, entry.file_size)?;

            // Use the original 8.3 name from the directory entry.
            // The first character is always '?' because FAT32 overwrites it
            // with 0xE5 on deletion — replace with '_' (standard convention).
            // Lowercase the result for filesystem compatibility.
            let filename = entry.name.replacen('?', "_", 1).to_lowercase();

            debug!(filename, bytes = bytes.len(), "file recovered via FAT32");
            extracted.push(ExtractedFile {
                filename,
                extension,
                bytes,
            });
        }

        info!(files_recovered = extracted.len(), "FAT32 recovery complete");
        Ok(extracted)
    }
}

/// Extracts the file extension from the 8.3 short name, lowercased.
/// Returns `None` if the name has no extension (no dot or empty extension).
fn extension_from_entry(entry: &DeletedEntry) -> Option<String> {
    let (_, ext) = entry.name.rsplit_once('.')?;
    let ext = ext.trim();
    if ext.is_empty() {
        return None;
    }
    Some(ext.to_lowercase())
}

/// Reads `file_size` bytes from the disk by walking clusters
/// **contiguously** starting at `first_cluster`, without consulting the FAT.
///
/// FAT entries for deleted files are zeroed out (freed), so the only
/// reliable information is the starting cluster stored in the directory entry.
/// Contiguous reading is the standard recovery heuristic for FAT32.
fn read_contiguous<R: Read + Seek + ?Sized>(
    source: &mut R,
    boot: &Fat32BootSector,
    first_cluster: u32,
    file_size: u32,
) -> Result<Vec<u8>, EngineError> {
    if file_size == 0 {
        return Ok(Vec::new());
    }

    let cluster_size = boot.cluster_size() as usize;
    let total_needed = file_size as usize;
    let clusters_needed = (total_needed + cluster_size - 1).div_ceil(cluster_size);

    let mut data = Vec::with_capacity(total_needed);

    for i in 0..clusters_needed {
        let cluster = first_cluster + i as u32;

        let offset =
            boot.cluster_offset(cluster)
                .ok_or_else(|| EngineError::InvalidFilesystem {
                    reason: format!(
                        "cluster {cluster} out of range during contiguous read of '{}'",
                        first_cluster
                    ),
                })?;

        source.seek(SeekFrom::Start(offset))?;

        let remaining = total_needed - data.len();
        let to_read = remaining.min(cluster_size);
        let mut buf = vec![0u8; to_read];
        source.read_exact(&mut buf)?;
        data.extend_from_slice(&buf);
    }

    Ok(data)
}

#[cfg(test)]
#[path = "fat32_strategy_tests.rs"]
mod tests;
