/// Builds a minimal in-memory FAT32 image for testing.
///
/// Layout chosen for simplicity:
///
/// ```text
/// Geometry
///   bytes_per_sector   = 512
///   sectors_per_cluster = 1          → cluster_size = 512 bytes
///   reserved_sectors   = 4           → fat_start at offset 2048
///   num_fats           = 1
///   fat_size_32        = 2 sectors   → data_start at offset 2048 + 1024 = 3072
///   root_cluster       = 2
///   total_sectors_32   = 16
///
/// Offsets
///   Boot sector: 0
///   FAT1:        2048  (2 sectors × 512 = 1024 bytes)
///   Cluster 2:   3072  (root directory, 512 bytes → 16 entries)
///   Cluster 3:   3584  (file data cluster)
///   …
///
/// Total image size: 16 × 512 = 8192 bytes
/// ```
///
/// See [`build_image_with_data`] to also write raw bytes into specific clusters.
pub fn build_image(fat_entries: &[(u32, u32)], dir_entries: &[([u8; 32], u32)]) -> Vec<u8> {
    build_image_with_data(fat_entries, dir_entries, &[])
}

/// Like [`build_image`] but also writes raw byte content into data clusters.
///
/// `data_clusters` is a slice of `(cluster_number, bytes)` pairs.  The bytes
/// are written verbatim to the start of the given cluster (truncated to 512
/// bytes if longer).  Use this to place real file content into an image so
/// that [`crate::strategies::Fat32Strategy`] can read it back.
pub fn build_image_with_data(
    fat_entries: &[(u32, u32)],
    dir_entries: &[([u8; 32], u32)],
    data_clusters: &[(u32, &[u8])],
) -> Vec<u8> {
    const BPS: u16 = 512;
    const SPC: u8 = 1;
    const RESERVED: u16 = 4;
    const NUM_FATS: u8 = 1;
    const FAT_SIZE: u32 = 2;
    const ROOT_CLUSTER: u32 = 2;
    const TOTAL_SECTORS: u32 = 16;

    let image_size = TOTAL_SECTORS as usize * BPS as usize;
    let mut img = vec![0u8; image_size];

    // ── Boot sector ──────────────────────────────────────────────────────────
    img[11..13].copy_from_slice(&BPS.to_le_bytes());
    img[13] = SPC;
    img[14..16].copy_from_slice(&RESERVED.to_le_bytes());
    img[16] = NUM_FATS;
    img[32..36].copy_from_slice(&TOTAL_SECTORS.to_le_bytes());
    img[36..40].copy_from_slice(&FAT_SIZE.to_le_bytes());
    img[44..48].copy_from_slice(&ROOT_CLUSTER.to_le_bytes());

    // ── FAT: mandatory entries 0 and 1 ───────────────────────────────────────
    let fat_start = RESERVED as usize * BPS as usize; // 2048
    write_fat_entry(&mut img, fat_start, 0, 0x0FFF_FFF8); // media marker
    write_fat_entry(&mut img, fat_start, 1, 0x0FFF_FFFF); // reserved

    for &(idx, value) in fat_entries {
        write_fat_entry(&mut img, fat_start, idx, value);
    }

    // ── Directory entries ─────────────────────────────────────────────────────
    let data_start = fat_start + NUM_FATS as usize * FAT_SIZE as usize * BPS as usize; // 3072
    let cluster_size = BPS as usize * SPC as usize; // 512

    for &(ref entry_bytes, cluster_idx) in dir_entries {
        let cluster_offset = data_start + (cluster_idx as usize - 2) * cluster_size;
        // Place in the first free 32-byte slot in that cluster.
        for slot in 0..(cluster_size / 32) {
            let slot_offset = cluster_offset + slot * 32;
            if img[slot_offset] == 0x00 {
                img[slot_offset..slot_offset + 32].copy_from_slice(entry_bytes);
                break;
            }
        }
    }

    // ── Raw data clusters ─────────────────────────────────────────────────────
    for &(cluster_idx, bytes) in data_clusters {
        let cluster_offset = data_start + (cluster_idx as usize - 2) * cluster_size;
        let len = bytes.len().min(cluster_size);
        img[cluster_offset..cluster_offset + len].copy_from_slice(&bytes[..len]);
    }

    img
}

fn write_fat_entry(img: &mut [u8], fat_start: usize, cluster: u32, value: u32) {
    let offset = fat_start + cluster as usize * 4;
    img[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// Returns the boot sector struct matching the geometry produced by
/// [`build_image`] / [`build_image_with_data`].
pub fn test_boot_sector() -> crate::filesystems::fat32::Fat32BootSector {
    crate::filesystems::fat32::Fat32BootSector {
        bytes_per_sector: 512,
        sectors_per_cluster: 1,
        reserved_sectors: 4,
        num_fats: 1,
        total_sectors_32: 16,
        fat_size_32: 2,
        root_cluster: 2,
    }
}

/// Builds a raw 32-byte directory entry.
///
/// * `name`       – 8 characters, space-padded (first byte `0xE5` = deleted)
/// * `ext`        – 3 characters, space-padded
/// * `attrs`      – attribute byte (`0x20` = archive, `0x0F` = LFN, …)
/// * `cluster`    – first data cluster (split into high/low 16-bit words)
/// * `file_size`  – file size in bytes
pub fn make_dir_entry(
    name: &[u8; 8],
    ext: &[u8; 3],
    attrs: u8,
    cluster: u32,
    file_size: u32,
) -> [u8; 32] {
    let mut raw = [0u8; 32];
    raw[0..8].copy_from_slice(name);
    raw[8..11].copy_from_slice(ext);
    raw[11] = attrs;
    raw[20..22].copy_from_slice(&((cluster >> 16) as u16).to_le_bytes());
    raw[26..28].copy_from_slice(&(cluster as u16).to_le_bytes());
    raw[28..32].copy_from_slice(&file_size.to_le_bytes());
    raw
}
