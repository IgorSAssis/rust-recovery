use std::io::Cursor;

use file_carver::carved_file::CarvedFile;
use file_carver::signature::{FileKind, JPEG_SIGNATURE, PNG_SIGNATURE};

use super::engine::{ExtractedFile, RecoveryEngine};

// ── in-memory disk builder ────────────────────────────────────────────────────

const SECTOR_SIZE: usize = 512;

/// Constructs a file whose bytes are: `header` + zero-filled body + `footer`.
/// Using 0x00 as filler guarantees no accidental signature matches.
fn make_file_bytes(header: &[u8], footer: &[u8], body_size: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(header);
    bytes.extend(vec![0x00u8; body_size]);
    bytes.extend_from_slice(footer);
    bytes
}

/// Pads `data` with zeros to reach exactly `total_size` bytes.
fn pad_to(mut data: Vec<u8>, total_size: usize) -> Vec<u8> {
    data.resize(total_size, 0x00);
    data
}

/// Builds a deterministic in-memory disk image with the following layout:
///
/// | Sector | Offset | Content                              |
/// |--------|--------|--------------------------------------|
/// |   0    |     0  | Filler (0x55, no signatures)         |
/// |   1-2  |   512  | JPEG 1 (header + body + footer)      |
/// |   3-4  |  1536  | PNG  1 (signature + body + IEND)     |
/// |   5-6  |  2560  | JPEG 2 (header + body + footer)      |
/// |   7    |  3584  | Corrupted JPEG (header only, no EOI) |
/// |   8    |  4096  | Zeros (end of disk)                  |
fn build_test_disk() -> Vec<u8> {
    let mut disk: Vec<u8> = Vec::new();

    disk.extend(vec![0x55u8; SECTOR_SIZE]);

    let jpeg1 = make_file_bytes(JPEG_SIGNATURE.header_pattern, JPEG_SIGNATURE.footer_pattern, 50);
    disk.extend(pad_to(jpeg1, 2 * SECTOR_SIZE));

    let png1 = make_file_bytes(PNG_SIGNATURE.header_pattern, PNG_SIGNATURE.footer_pattern, 80);
    disk.extend(pad_to(png1, 2 * SECTOR_SIZE));

    let jpeg2 = make_file_bytes(JPEG_SIGNATURE.header_pattern, JPEG_SIGNATURE.footer_pattern, 70);
    disk.extend(pad_to(jpeg2, 2 * SECTOR_SIZE));

    let corrupted = JPEG_SIGNATURE.header_pattern.to_vec();
    disk.extend(pad_to(corrupted, SECTOR_SIZE));

    disk.extend(vec![0x00u8; SECTOR_SIZE]);

    disk
}

// ── scan tests (fully in-memory) ──────────────────────────────────────────────

#[test]
fn scan_finds_all_recoverable_files() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::new(std::env::temp_dir()).with_chunk_size(512);

    let carved_files = engine.scan(&mut source).unwrap();

    assert_eq!(carved_files.len(), 3, "expected 3 recoverable files");
}

#[test]
fn scan_returns_files_at_correct_offsets() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::new(std::env::temp_dir()).with_chunk_size(512);

    let carved_files = engine.scan(&mut source).unwrap();

    assert_eq!(carved_files[0].offset_start, 512);
    assert_eq!(carved_files[1].offset_start, 1536);
    assert_eq!(carved_files[2].offset_start, 2560);
}

#[test]
fn scan_returns_files_with_correct_kinds() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::new(std::env::temp_dir()).with_chunk_size(512);

    let carved_files = engine.scan(&mut source).unwrap();

    assert_eq!(carved_files[0].kind, FileKind::Jpeg);
    assert_eq!(carved_files[1].kind, FileKind::Png);
    assert_eq!(carved_files[2].kind, FileKind::Jpeg);
}

#[test]
fn scan_ignores_corrupted_file_without_footer() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::new(std::env::temp_dir()).with_chunk_size(512);

    let carved_files = engine.scan(&mut source).unwrap();

    let starts: Vec<u64> = carved_files.iter().map(|carved| carved.offset_start).collect();
    assert!(!starts.contains(&3584), "corrupted file should not appear in results");
}

// ── extract_all tests (fully in-memory) ───────────────────────────────────────

#[test]
fn extract_all_returns_one_entry_per_carved_file() {
    let disk = build_test_disk();
    let engine = RecoveryEngine::new(std::env::temp_dir()).with_chunk_size(512);

    let mut source = Cursor::new(disk.clone());
    let carved_files = engine.scan(&mut source).unwrap();

    let mut source = Cursor::new(disk);
    let extracted = engine.extract_all(&mut source, &carved_files).unwrap();

    assert_eq!(extracted.len(), carved_files.len());
}

#[test]
fn extract_all_returns_files_with_correct_kinds() {
    let disk = build_test_disk();
    let engine = RecoveryEngine::new(std::env::temp_dir()).with_chunk_size(512);

    let mut source = Cursor::new(disk.clone());
    let carved_files = engine.scan(&mut source).unwrap();

    let mut source = Cursor::new(disk);
    let extracted = engine.extract_all(&mut source, &carved_files).unwrap();

    assert_eq!(extracted[0].kind, FileKind::Jpeg);
    assert_eq!(extracted[1].kind, FileKind::Png);
    assert_eq!(extracted[2].kind, FileKind::Jpeg);
}

#[test]
fn extract_all_returns_files_with_correct_filenames() {
    let disk = build_test_disk();
    let engine = RecoveryEngine::new(std::env::temp_dir()).with_chunk_size(512);

    let mut source = Cursor::new(disk.clone());
    let carved_files = engine.scan(&mut source).unwrap();

    let mut source = Cursor::new(disk);
    let extracted = engine.extract_all(&mut source, &carved_files).unwrap();

    assert_eq!(extracted[0].filename, "recovered_0.jpg");
    assert_eq!(extracted[1].filename, "recovered_1.png");
    assert_eq!(extracted[2].filename, "recovered_2.jpg");
}

#[test]
fn extract_all_bytes_start_with_correct_header() {
    let disk = build_test_disk();
    let engine = RecoveryEngine::new(std::env::temp_dir()).with_chunk_size(512);

    let mut source = Cursor::new(disk.clone());
    let carved_files = engine.scan(&mut source).unwrap();

    let mut source = Cursor::new(disk);
    let extracted = engine.extract_all(&mut source, &carved_files).unwrap();

    assert!(extracted[0].bytes.starts_with(JPEG_SIGNATURE.header_pattern));
    assert!(extracted[1].bytes.starts_with(PNG_SIGNATURE.header_pattern));
    assert!(extracted[2].bytes.starts_with(JPEG_SIGNATURE.header_pattern));
}

#[test]
fn extract_all_bytes_end_with_correct_footer() {
    let disk = build_test_disk();
    let engine = RecoveryEngine::new(std::env::temp_dir()).with_chunk_size(512);

    let mut source = Cursor::new(disk.clone());
    let carved_files = engine.scan(&mut source).unwrap();

    let mut source = Cursor::new(disk);
    let extracted = engine.extract_all(&mut source, &carved_files).unwrap();

    assert!(extracted[0].bytes.ends_with(JPEG_SIGNATURE.footer_pattern));
    assert!(extracted[1].bytes.ends_with(PNG_SIGNATURE.footer_pattern));
    assert!(extracted[2].bytes.ends_with(JPEG_SIGNATURE.footer_pattern));
}

#[test]
fn extract_all_with_empty_carved_list_returns_empty_vec() {
    let engine = RecoveryEngine::new(std::env::temp_dir());
    let mut source = Cursor::new(build_test_disk());

    let extracted = engine.extract_all(&mut source, &[]).unwrap();

    assert!(extracted.is_empty());
}

#[test]
fn extract_all_with_manually_constructed_carved_file_extracts_correct_bytes() {
    let mut disk = vec![0xAAu8; 1024];
    disk[100..103].copy_from_slice(JPEG_SIGNATURE.header_pattern);
    disk[107..109].copy_from_slice(JPEG_SIGNATURE.footer_pattern);

    let carved_file = CarvedFile {
        kind: FileKind::Jpeg,
        offset_start: 100,
        offset_end: 109,
    };

    let engine = RecoveryEngine::new(std::env::temp_dir());
    let mut source = Cursor::new(disk.clone());
    let extracted = engine.extract_all(&mut source, &[carved_file]).unwrap();

    assert_eq!(extracted[0].bytes, &disk[100..109]);
}

// ── save_all tests (filesystem — isolated to this section) ───────────────────

fn temp_output_dir(suffix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rust_recovery_out_{}", suffix));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn save_all_creates_output_directory_and_files() {
    let output_dir = temp_output_dir("save_all");
    let engine = RecoveryEngine::new(&output_dir);

    let extracted = vec![
        ExtractedFile {
            filename: "recovered_0.jpg".to_string(),
            kind: FileKind::Jpeg,
            bytes: JPEG_SIGNATURE.header_pattern.to_vec(),
        },
        ExtractedFile {
            filename: "recovered_1.png".to_string(),
            kind: FileKind::Png,
            bytes: PNG_SIGNATURE.header_pattern.to_vec(),
        },
    ];

    let saved_paths = engine.save_all(&extracted).unwrap();

    assert_eq!(saved_paths.len(), 2);
    assert!(saved_paths[0].exists());
    assert!(saved_paths[1].exists());
}

#[test]
fn save_all_writes_correct_bytes_to_disk() {
    let output_dir = temp_output_dir("save_bytes");
    let engine = RecoveryEngine::new(&output_dir);

    let expected_bytes = vec![0x01, 0x02, 0x03, 0x04];
    let extracted = vec![ExtractedFile {
        filename: "recovered_0.jpg".to_string(),
        kind: FileKind::Jpeg,
        bytes: expected_bytes.clone(),
    }];

    let saved_paths = engine.save_all(&extracted).unwrap();
    let written = std::fs::read(&saved_paths[0]).unwrap();

    assert_eq!(written, expected_bytes);
}
