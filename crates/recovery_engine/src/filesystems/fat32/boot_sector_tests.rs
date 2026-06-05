use std::io::Cursor;

use super::Fat32BootSector;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Builds a minimal valid 512-byte FAT32 boot sector buffer with the given
/// field values.  All fields not listed are left as `0x00`.
fn make_raw(
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    num_fats: u8,
    total_sectors_32: u32,
    fat_size_32: u32,
    root_cluster: u32,
) -> [u8; 512] {
    let mut raw = [0u8; 512];
    raw[11..13].copy_from_slice(&bytes_per_sector.to_le_bytes());
    raw[13] = sectors_per_cluster;
    raw[14..16].copy_from_slice(&reserved_sectors.to_le_bytes());
    raw[16] = num_fats;
    raw[32..36].copy_from_slice(&total_sectors_32.to_le_bytes());
    raw[36..40].copy_from_slice(&fat_size_32.to_le_bytes());
    raw[44..48].copy_from_slice(&root_cluster.to_le_bytes());
    raw
}

/// A canonical valid FAT32 boot sector representing a 10 MiB volume:
///
/// - 512 bytes/sector, 8 sectors/cluster → 4096-byte clusters
/// - 32 reserved sectors, 2 FATs, fat_size = 20 sectors each
/// - total_sectors = 20480 (10 MiB / 512)
/// - root_cluster = 2
fn default_raw() -> [u8; 512] {
    make_raw(512, 8, 32, 2, 20480, 20, 2)
}

// ── field extraction ──────────────────────────────────────────────────────────

#[test]
fn parse_bytes_extracts_bytes_per_sector() {
    assert_eq!(
        Fat32BootSector::parse_bytes(&default_raw())
            .unwrap()
            .bytes_per_sector,
        512
    );
}

#[test]
fn parse_bytes_extracts_sectors_per_cluster() {
    assert_eq!(
        Fat32BootSector::parse_bytes(&default_raw())
            .unwrap()
            .sectors_per_cluster,
        8
    );
}

#[test]
fn parse_bytes_extracts_reserved_sectors() {
    assert_eq!(
        Fat32BootSector::parse_bytes(&default_raw())
            .unwrap()
            .reserved_sectors,
        32
    );
}

#[test]
fn parse_bytes_extracts_num_fats() {
    assert_eq!(
        Fat32BootSector::parse_bytes(&default_raw())
            .unwrap()
            .num_fats,
        2
    );
}

#[test]
fn parse_bytes_extracts_total_sectors_32() {
    assert_eq!(
        Fat32BootSector::parse_bytes(&default_raw())
            .unwrap()
            .total_sectors_32,
        20480
    );
}

#[test]
fn parse_bytes_extracts_fat_size_32() {
    assert_eq!(
        Fat32BootSector::parse_bytes(&default_raw())
            .unwrap()
            .fat_size_32,
        20
    );
}

#[test]
fn parse_bytes_extracts_root_cluster() {
    assert_eq!(
        Fat32BootSector::parse_bytes(&default_raw())
            .unwrap()
            .root_cluster,
        2
    );
}

// ── derived computations ──────────────────────────────────────────────────────

#[test]
fn cluster_size_is_bytes_per_sector_times_sectors_per_cluster() {
    // 512 * 8 = 4096
    assert_eq!(
        Fat32BootSector::parse_bytes(&default_raw())
            .unwrap()
            .cluster_size(),
        4096
    );
}

#[test]
fn fat_start_offset_is_reserved_sectors_times_bytes_per_sector() {
    // 32 * 512 = 16384
    assert_eq!(
        Fat32BootSector::parse_bytes(&default_raw())
            .unwrap()
            .fat_start_offset(),
        16384
    );
}

#[test]
fn data_start_offset_accounts_for_both_fats() {
    // fat_start(16384) + 2 fats × 20 sectors × 512 bytes = 16384 + 20480 = 36864
    assert_eq!(
        Fat32BootSector::parse_bytes(&default_raw())
            .unwrap()
            .data_start_offset(),
        36864
    );
}

#[test]
fn cluster_offset_for_cluster_2_equals_data_start() {
    let boot = Fat32BootSector::parse_bytes(&default_raw()).unwrap();
    assert_eq!(boot.cluster_offset(2), Some(boot.data_start_offset()));
}

#[test]
fn cluster_offset_for_cluster_3_is_one_cluster_size_past_data_start() {
    let boot = Fat32BootSector::parse_bytes(&default_raw()).unwrap();
    assert_eq!(
        boot.cluster_offset(3),
        Some(boot.data_start_offset() + boot.cluster_size())
    );
}

#[test]
fn cluster_offset_returns_none_for_cluster_0() {
    let boot = Fat32BootSector::parse_bytes(&default_raw()).unwrap();
    assert_eq!(boot.cluster_offset(0), None);
}

#[test]
fn cluster_offset_returns_none_for_cluster_1() {
    let boot = Fat32BootSector::parse_bytes(&default_raw()).unwrap();
    assert_eq!(boot.cluster_offset(1), None);
}

// ── validation ────────────────────────────────────────────────────────────────

#[test]
fn validate_rejects_zero_bytes_per_sector() {
    assert!(Fat32BootSector::parse_bytes(&make_raw(0, 8, 32, 2, 20480, 20, 2)).is_err());
}

#[test]
fn validate_rejects_non_power_of_two_sectors_per_cluster() {
    assert!(Fat32BootSector::parse_bytes(&make_raw(512, 3, 32, 2, 20480, 20, 2)).is_err());
}

#[test]
fn validate_rejects_zero_num_fats() {
    assert!(Fat32BootSector::parse_bytes(&make_raw(512, 8, 32, 0, 20480, 20, 2)).is_err());
}

#[test]
fn validate_rejects_zero_fat_size_32() {
    assert!(Fat32BootSector::parse_bytes(&make_raw(512, 8, 32, 2, 20480, 0, 2)).is_err());
}

// ── parse via Read + Seek ─────────────────────────────────────────────────────

#[test]
fn parse_reads_from_cursor_seeking_to_offset_0() {
    let raw = default_raw();
    let mut cursor = Cursor::new(raw.to_vec());
    // Advance the cursor to a non-zero position to verify `parse` seeks back.
    cursor.set_position(100);
    let boot = Fat32BootSector::parse(&mut cursor).unwrap();
    assert_eq!(boot.bytes_per_sector, 512);
    assert_eq!(boot.root_cluster, 2);
}
