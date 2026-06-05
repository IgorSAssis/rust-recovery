use std::io::Cursor;

use super::{DELETED_MARKER, DeletedEntry, Fat32RawDirEntry, list_deleted_entries};
use crate::filesystems::fat32::test_helpers::{build_image, make_dir_entry};

// ── Fat32RawDirEntry — field detection ────────────────────────────────────────

#[test]
fn is_deleted_returns_true_when_first_byte_is_0xe5() {
    let mut raw = [0u8; 32];
    raw[0] = DELETED_MARKER;
    raw[11] = 0x20; // archive
    assert!(Fat32RawDirEntry::from_raw(raw).is_deleted());
}

#[test]
fn is_deleted_returns_false_for_normal_entry() {
    let raw = make_dir_entry(b"README  ", b"TXT", 0x20, 3, 100);
    assert!(!Fat32RawDirEntry::from_raw(raw).is_deleted());
}

#[test]
fn is_end_of_dir_returns_true_when_first_byte_is_zero() {
    let raw = [0u8; 32];
    assert!(Fat32RawDirEntry::from_raw(raw).is_end_of_dir());
}

#[test]
fn is_end_of_dir_returns_false_for_normal_entry() {
    let raw = make_dir_entry(b"HELLO   ", b"TXT", 0x20, 4, 20);
    assert!(!Fat32RawDirEntry::from_raw(raw).is_end_of_dir());
}

#[test]
fn is_long_name_entry_returns_true_for_0x0f_attributes() {
    let mut raw = [b'A'; 32];
    raw[11] = 0x0F;
    assert!(Fat32RawDirEntry::from_raw(raw).is_long_name_entry());
}

#[test]
fn is_long_name_entry_returns_false_for_normal_attributes() {
    let raw = make_dir_entry(b"FILE    ", b"BIN", 0x20, 5, 512);
    assert!(!Fat32RawDirEntry::from_raw(raw).is_long_name_entry());
}

#[test]
fn is_volume_label_returns_true_when_volume_bit_set() {
    let mut raw = [0u8; 32];
    raw[0] = b'V';
    raw[11] = 0x08; // volume label attribute
    assert!(Fat32RawDirEntry::from_raw(raw).is_volume_label());
}

#[test]
fn is_volume_label_returns_false_for_lfn_entry() {
    let mut raw = [b'X'; 32];
    raw[11] = 0x0F; // LFN, not volume label
    assert!(!Fat32RawDirEntry::from_raw(raw).is_volume_label());
}

// ── Fat32RawDirEntry — cluster and size ───────────────────────────────────────

#[test]
fn first_cluster_combines_high_and_low_words() {
    // high = 0x0001 at offset 20, low = 0x0005 at offset 26
    // result = (0x0001 << 16) | 0x0005 = 0x00010005
    let raw = make_dir_entry(b"BIG     ", b"BIN", 0x20, 0x0001_0005, 1024);
    let entry = Fat32RawDirEntry::from_raw(raw);
    assert_eq!(entry.first_cluster(), 0x0001_0005);
}

#[test]
fn first_cluster_returns_correct_simple_value() {
    let raw = make_dir_entry(b"SMALL   ", b"DAT", 0x20, 7, 256);
    assert_eq!(Fat32RawDirEntry::from_raw(raw).first_cluster(), 7);
}

#[test]
fn file_size_returns_correct_value() {
    let raw = make_dir_entry(b"DOC     ", b"PDF", 0x20, 3, 98_304);
    assert_eq!(Fat32RawDirEntry::from_raw(raw).file_size(), 98_304);
}

#[test]
fn file_size_returns_zero_for_empty_file() {
    let raw = make_dir_entry(b"EMPTY   ", b"   ", 0x20, 2, 0);
    assert_eq!(Fat32RawDirEntry::from_raw(raw).file_size(), 0);
}

// ── Fat32RawDirEntry — short_name ─────────────────────────────────────────────

#[test]
fn short_name_formats_name_and_extension() {
    let raw = make_dir_entry(b"README  ", b"TXT", 0x20, 3, 100);
    assert_eq!(Fat32RawDirEntry::from_raw(raw).short_name(), "README.TXT");
}

#[test]
fn short_name_trims_trailing_spaces_from_name() {
    let raw = make_dir_entry(b"FILE    ", b"BIN", 0x20, 4, 10);
    assert_eq!(Fat32RawDirEntry::from_raw(raw).short_name(), "FILE.BIN");
}

#[test]
fn short_name_omits_dot_when_extension_is_blank() {
    let raw = make_dir_entry(b"MAKEFILE", b"   ", 0x20, 5, 512);
    assert_eq!(Fat32RawDirEntry::from_raw(raw).short_name(), "MAKEFILE");
}

#[test]
fn short_name_replaces_deleted_marker_with_question_mark() {
    let mut raw = make_dir_entry(b"PHOTO   ", b"JPG", 0x20, 6, 2048);
    raw[0] = DELETED_MARKER; // simulate deletion
    assert_eq!(Fat32RawDirEntry::from_raw(raw).short_name(), "?HOTO.JPG");
}

// ── list_deleted_entries ──────────────────────────────────────────────────────

/// Cluster 2 = root dir (1 sector = 512 bytes → 16 slots).
/// FAT[2] = EOC (single-cluster root dir).
fn root_cluster() -> u32 {
    2
}

#[test]
fn list_deleted_entries_returns_empty_for_clean_directory() {
    // Only an end-of-dir marker in slot 0 of cluster 2.
    let img = build_image(&[(root_cluster(), 0x0FFF_FFFF)], &[]);
    let mut cur = Cursor::new(img);

    let boot = crate::filesystems::fat32::Fat32BootSector {
        bytes_per_sector: 512,
        sectors_per_cluster: 1,
        reserved_sectors: 4,
        num_fats: 1,
        total_sectors_32: 16,
        fat_size_32: 2,
        root_cluster: 2,
    };

    let result = super::list_deleted_entries(&mut cur, &boot).unwrap();
    assert!(result.is_empty());
}

#[test]
fn list_deleted_entries_finds_single_deleted_file() {
    let mut deleted_entry = make_dir_entry(b"PHOTO   ", b"JPG", 0x20, 4, 2048);
    deleted_entry[0] = DELETED_MARKER;

    let img = build_image(
        &[(root_cluster(), 0x0FFF_FFFF)],
        &[(deleted_entry, root_cluster())],
    );
    let mut cur = Cursor::new(img);

    let boot = crate::filesystems::fat32::Fat32BootSector {
        bytes_per_sector: 512,
        sectors_per_cluster: 1,
        reserved_sectors: 4,
        num_fats: 1,
        total_sectors_32: 16,
        fat_size_32: 2,
        root_cluster: 2,
    };

    let result = super::list_deleted_entries(&mut cur, &boot).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0],
        DeletedEntry {
            name: "?HOTO.JPG".into(),
            first_cluster: 4,
            file_size: 2048,
        }
    );
}

#[test]
fn list_deleted_entries_finds_multiple_deleted_files() {
    let mut entry_a = make_dir_entry(b"REPORT  ", b"PDF", 0x20, 5, 4096);
    entry_a[0] = DELETED_MARKER;
    let mut entry_b = make_dir_entry(b"IMAGE   ", b"PNG", 0x20, 6, 8192);
    entry_b[0] = DELETED_MARKER;

    let img = build_image(
        &[(root_cluster(), 0x0FFF_FFFF)],
        &[(entry_a, root_cluster()), (entry_b, root_cluster())],
    );
    let mut cur = Cursor::new(img);

    let boot = crate::filesystems::fat32::Fat32BootSector {
        bytes_per_sector: 512,
        sectors_per_cluster: 1,
        reserved_sectors: 4,
        num_fats: 1,
        total_sectors_32: 16,
        fat_size_32: 2,
        root_cluster: 2,
    };

    let result = super::list_deleted_entries(&mut cur, &boot).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "?EPORT.PDF");
    assert_eq!(result[1].name, "?MAGE.PNG");
}

#[test]
fn list_deleted_entries_skips_live_files() {
    let live_entry = make_dir_entry(b"LIVE    ", b"TXT", 0x20, 3, 100);
    let mut dead_entry = make_dir_entry(b"DEAD    ", b"TXT", 0x20, 4, 200);
    dead_entry[0] = DELETED_MARKER;

    let img = build_image(
        &[(root_cluster(), 0x0FFF_FFFF)],
        &[(live_entry, root_cluster()), (dead_entry, root_cluster())],
    );
    let mut cur = Cursor::new(img);

    let boot = crate::filesystems::fat32::Fat32BootSector {
        bytes_per_sector: 512,
        sectors_per_cluster: 1,
        reserved_sectors: 4,
        num_fats: 1,
        total_sectors_32: 16,
        fat_size_32: 2,
        root_cluster: 2,
    };

    let result = super::list_deleted_entries(&mut cur, &boot).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "?EAD.TXT");
}

#[test]
fn list_deleted_entries_stops_at_end_of_dir_marker() {
    let mut entry_a = make_dir_entry(b"BEFORE  ", b"BIN", 0x20, 5, 512);
    entry_a[0] = DELETED_MARKER;

    // entry_b is all zeros → end-of-dir; place after entry_a.
    // entry_c would be after the end marker and must NOT be returned.
    let end_of_dir = [0u8; 32];
    let mut entry_c = make_dir_entry(b"AFTER   ", b"BIN", 0x20, 6, 512);
    entry_c[0] = DELETED_MARKER;

    // Build manually: entry_a at slot 0, end_of_dir at slot 1, entry_c at slot 2.
    let mut img = build_image(
        &[(root_cluster(), 0x0FFF_FFFF)],
        &[(entry_a, root_cluster())],
    );

    // fat_start=2048, data_start=3072, cluster 2 starts at 3072.
    // Slot 1 = 3072 + 32 = 3104, Slot 2 = 3072 + 64 = 3136.
    img[3104..3136].copy_from_slice(&end_of_dir);
    img[3136..3168].copy_from_slice(&entry_c);

    let mut cur = Cursor::new(img);
    let boot = crate::filesystems::fat32::Fat32BootSector {
        bytes_per_sector: 512,
        sectors_per_cluster: 1,
        reserved_sectors: 4,
        num_fats: 1,
        total_sectors_32: 16,
        fat_size_32: 2,
        root_cluster: 2,
    };

    let result = super::list_deleted_entries(&mut cur, &boot).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "?EFORE.BIN");
}

#[test]
fn list_deleted_entries_skips_lfn_and_volume_label_entries() {
    let lfn_entry: [u8; 32] = {
        let mut e = [b'X'; 32];
        e[11] = 0x0F;
        e
    };
    let volume_label: [u8; 32] = {
        let mut e = [0u8; 32];
        e[0] = b'V';
        e[11] = 0x08;
        e
    };
    let mut deleted_entry = make_dir_entry(b"ACTUAL  ", b"DAT", 0x20, 4, 100);
    deleted_entry[0] = DELETED_MARKER;

    let img = build_image(
        &[(root_cluster(), 0x0FFF_FFFF)],
        &[
            (lfn_entry, root_cluster()),
            (volume_label, root_cluster()),
            (deleted_entry, root_cluster()),
        ],
    );
    let mut cur = Cursor::new(img);

    let boot = crate::filesystems::fat32::Fat32BootSector {
        bytes_per_sector: 512,
        sectors_per_cluster: 1,
        reserved_sectors: 4,
        num_fats: 1,
        total_sectors_32: 16,
        fat_size_32: 2,
        root_cluster: 2,
    };

    let result = super::list_deleted_entries(&mut cur, &boot).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "?CTUAL.DAT");
}
