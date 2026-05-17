use std::io::Cursor;

use crate::filesystems::fat32::dir_entry::DELETED_MARKER;
use crate::filesystems::fat32::test_helpers::{build_image_with_data, make_dir_entry};
use crate::strategies::RecoveryStrategy;

use super::Fat32Strategy;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Minimal JPEG: SOI marker + EOI marker (4 bytes).
/// Enough for FileKind detection to round-trip; not a valid JPEG image.
const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, b'J', b'P', b'E', b'G'];

/// Minimal PNG: 8-byte magic header.
const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

// ── single deleted JPEG ───────────────────────────────────────────────────────

#[test]
fn recover_returns_single_deleted_jpeg() {
    // root dir (cluster 2) → EOC
    // file data lives in cluster 3
    let mut file_content = vec![0u8; 512];
    file_content[..JPEG_MAGIC.len()].copy_from_slice(JPEG_MAGIC);

    let mut dir_entry = make_dir_entry(b"PHOTO   ", b"JPG", 0x20, 3, 8);
    dir_entry[0] = DELETED_MARKER;

    let img = build_image_with_data(
        &[(2, 0x0FFF_FFFF)], // root dir cluster 2 = EOC
        &[(dir_entry, 2)],
        &[(3, &file_content)],
    );

    let mut cursor = Cursor::new(img);
    let result = Fat32Strategy::new().recover(&mut cursor).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].filename, "recovered_fat32_0.jpg");
    assert_eq!(&result[0].bytes, &JPEG_MAGIC[..8]);
}

#[test]
fn recover_returns_correct_file_size() {
    let mut file_content = vec![0xAB_u8; 512];
    file_content[0] = 0xFF;
    file_content[1] = 0xD8;
    file_content[2] = 0xFF;

    let file_size: u32 = 300;
    let mut dir_entry = make_dir_entry(b"IMG     ", b"JPG", 0x20, 3, file_size);
    dir_entry[0] = DELETED_MARKER;

    let img = build_image_with_data(
        &[(2, 0x0FFF_FFFF)],
        &[(dir_entry, 2)],
        &[(3, &file_content)],
    );

    let mut cursor = Cursor::new(img);
    let result = Fat32Strategy::new().recover(&mut cursor).unwrap();

    assert_eq!(result[0].bytes.len(), 300);
    // First 3 bytes are the JPEG-like header we wrote
    assert_eq!(&result[0].bytes[..3], &[0xFF, 0xD8, 0xFF]);
}

// ── multiple deleted files ────────────────────────────────────────────────────

#[test]
fn recover_returns_multiple_deleted_files() {
    let mut jpeg_data = vec![0u8; 512];
    jpeg_data[..JPEG_MAGIC.len()].copy_from_slice(JPEG_MAGIC);

    let mut png_data = vec![0u8; 512];
    png_data[..PNG_MAGIC.len()].copy_from_slice(PNG_MAGIC);

    let mut jpeg_entry = make_dir_entry(b"PHOTO   ", b"JPG", 0x20, 3, 8);
    jpeg_entry[0] = DELETED_MARKER;

    let mut png_entry = make_dir_entry(b"IMAGE   ", b"PNG", 0x20, 4, 8);
    png_entry[0] = DELETED_MARKER;

    let img = build_image_with_data(
        &[(2, 0x0FFF_FFFF)],
        &[(jpeg_entry, 2), (png_entry, 2)],
        &[(3, &jpeg_data), (4, &png_data)],
    );

    let mut cursor = Cursor::new(img);
    let result = Fat32Strategy::new().recover(&mut cursor).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].filename, "recovered_fat32_0.jpg");
    assert_eq!(result[1].filename, "recovered_fat32_1.png");
}

// ── skipping rules ────────────────────────────────────────────────────────────

#[test]
fn recover_skips_entries_with_unknown_extensions() {
    // ".DAT" is not in the known FileKind set
    let mut entry = make_dir_entry(b"BINARY  ", b"DAT", 0x20, 3, 100);
    entry[0] = DELETED_MARKER;

    let img = build_image_with_data(
        &[(2, 0x0FFF_FFFF)],
        &[(entry, 2)],
        &[],
    );

    let mut cursor = Cursor::new(img);
    let result = Fat32Strategy::new().recover(&mut cursor).unwrap();
    assert!(result.is_empty());
}

#[test]
fn recover_skips_live_files_and_returns_only_deleted() {
    let live_entry = make_dir_entry(b"LIVE    ", b"JPG", 0x20, 3, 8);
    let mut dead_entry = make_dir_entry(b"DEAD    ", b"PNG", 0x20, 4, 8);
    dead_entry[0] = DELETED_MARKER;

    let mut png_data = vec![0u8; 512];
    png_data[..PNG_MAGIC.len()].copy_from_slice(PNG_MAGIC);

    let img = build_image_with_data(
        &[(2, 0x0FFF_FFFF)],
        &[(live_entry, 2), (dead_entry, 2)],
        &[(4, &png_data)],
    );

    let mut cursor = Cursor::new(img);
    let result = Fat32Strategy::new().recover(&mut cursor).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].filename, "recovered_fat32_0.png");
}

#[test]
fn recover_returns_empty_for_clean_directory() {
    let img = build_image_with_data(&[(2, 0x0FFF_FFFF)], &[], &[]);
    let mut cursor = Cursor::new(img);
    let result = Fat32Strategy::new().recover(&mut cursor).unwrap();
    assert!(result.is_empty());
}

// ── invalid boot sector ───────────────────────────────────────────────────────

#[test]
fn recover_returns_error_for_invalid_boot_sector() {
    // All-zero image → bytes_per_sector = 0 → InvalidFilesystem error
    let img = vec![0u8; 8192];
    let mut cursor = Cursor::new(img);
    let result = Fat32Strategy::new().recover(&mut cursor);
    assert!(result.is_err());
}

// ── engine integration ────────────────────────────────────────────────────────

#[test]
fn recovery_engine_with_fat32_strategy_recovers_deleted_file() {
    use crate::engine::RecoveryEngine;
    use crate::strategies::Fat32Strategy;

    let mut jpeg_data = vec![0u8; 512];
    jpeg_data[..JPEG_MAGIC.len()].copy_from_slice(JPEG_MAGIC);

    let mut dir_entry = make_dir_entry(b"PHOTO   ", b"JPG", 0x20, 3, 8);
    dir_entry[0] = DELETED_MARKER;

    let img = build_image_with_data(
        &[(2, 0x0FFF_FFFF)],
        &[(dir_entry, 2)],
        &[(3, &jpeg_data)],
    );

    let engine = RecoveryEngine::new()
        .with_strategy(Box::new(Fat32Strategy::new()));

    let mut cursor = Cursor::new(img);
    let extracted = engine.recover(&mut cursor).unwrap();

    assert_eq!(extracted.len(), 1);
    assert_eq!(&extracted[0].bytes, JPEG_MAGIC);
}
