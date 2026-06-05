use std::io::Cursor;

use file_carver::constants::DEFAULT_CHUNK_SIZE;
use file_carver::signature::SUPPORTED_SIGNATURES;
use file_carver::signature::{JPEG_SIGNATURE, PNG_SIGNATURE};

use super::engine::RecoveryEngine;
use super::error::EngineError;
use super::types::ExtractedFile;

// ── in-memory disk builder ────────────────────────────────────────────────────

const SECTOR_SIZE: usize = 512;

fn make_file_bytes(header: &[u8], footer: &[u8], body_size: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(header);
    bytes.extend(vec![0x00u8; body_size]);
    bytes.extend_from_slice(footer);
    bytes
}

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

    let jpeg1 = make_file_bytes(
        JPEG_SIGNATURE.header_pattern,
        JPEG_SIGNATURE.footer_pattern,
        50,
    );
    disk.extend(pad_to(jpeg1, 2 * SECTOR_SIZE));

    let png1 = make_file_bytes(
        PNG_SIGNATURE.header_pattern,
        PNG_SIGNATURE.footer_pattern,
        80,
    );
    disk.extend(pad_to(png1, 2 * SECTOR_SIZE));

    let jpeg2 = make_file_bytes(
        JPEG_SIGNATURE.header_pattern,
        JPEG_SIGNATURE.footer_pattern,
        70,
    );
    disk.extend(pad_to(jpeg2, 2 * SECTOR_SIZE));

    let corrupted = JPEG_SIGNATURE.header_pattern.to_vec();
    disk.extend(pad_to(corrupted, SECTOR_SIZE));

    disk.extend(vec![0x00u8; SECTOR_SIZE]);

    disk
}

// ── scan tests ────────────────────────────────────────────────────────────────

#[test]
fn scan_finds_all_recoverable_files() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::for_carver(SUPPORTED_SIGNATURES.iter().collect(), SECTOR_SIZE);

    let file_infos = engine.scan(&mut source).unwrap();

    assert_eq!(file_infos.len(), 3, "expected 3 recoverable files");
}

#[test]
fn scan_returns_files_with_correct_extensions() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::for_carver(SUPPORTED_SIGNATURES.iter().collect(), SECTOR_SIZE);

    let file_infos = engine.scan(&mut source).unwrap();

    assert_eq!(file_infos[0].extension, "jpg");
    assert_eq!(file_infos[1].extension, "png");
    assert_eq!(file_infos[2].extension, "jpg");
}

#[test]
fn scan_returns_files_with_correct_filenames() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::for_carver(SUPPORTED_SIGNATURES.iter().collect(), SECTOR_SIZE);

    let file_infos = engine.scan(&mut source).unwrap();

    assert_eq!(file_infos[0].filename, "recovered_0.jpg");
    assert_eq!(file_infos[1].filename, "recovered_1.png");
    assert_eq!(file_infos[2].filename, "recovered_2.jpg");
}

#[test]
fn scan_ignores_corrupted_file_without_footer() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::for_carver(SUPPORTED_SIGNATURES.iter().collect(), SECTOR_SIZE);

    let file_infos = engine.scan(&mut source).unwrap();

    assert_eq!(
        file_infos.len(),
        3,
        "corrupted file without footer must not appear in results"
    );
}

#[test]
fn scan_without_output_dir_succeeds() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::for_carver(SUPPORTED_SIGNATURES.iter().collect(), SECTOR_SIZE);

    let file_infos = engine.scan(&mut source).unwrap();

    assert_eq!(file_infos.len(), 3);
}

// ── recover tests ─────────────────────────────────────────────────────────────

#[test]
fn recover_returns_one_entry_per_file() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::for_carver(SUPPORTED_SIGNATURES.iter().collect(), SECTOR_SIZE);

    let extracted = engine.recover(&mut source).unwrap();

    assert_eq!(extracted.len(), 3);
}

#[test]
fn recover_returns_files_with_correct_extensions() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::for_carver(SUPPORTED_SIGNATURES.iter().collect(), SECTOR_SIZE);

    let extracted = engine.recover(&mut source).unwrap();

    assert_eq!(extracted[0].extension, "jpg");
    assert_eq!(extracted[1].extension, "png");
    assert_eq!(extracted[2].extension, "jpg");
}

#[test]
fn recover_returns_files_with_correct_filenames() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::for_carver(SUPPORTED_SIGNATURES.iter().collect(), SECTOR_SIZE);

    let extracted = engine.recover(&mut source).unwrap();

    assert_eq!(extracted[0].filename, "recovered_0.jpg");
    assert_eq!(extracted[1].filename, "recovered_1.png");
    assert_eq!(extracted[2].filename, "recovered_2.jpg");
}

#[test]
fn recover_bytes_start_with_correct_header() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::for_carver(SUPPORTED_SIGNATURES.iter().collect(), SECTOR_SIZE);

    let extracted = engine.recover(&mut source).unwrap();

    assert!(
        extracted[0]
            .bytes
            .starts_with(JPEG_SIGNATURE.header_pattern)
    );
    assert!(extracted[1].bytes.starts_with(PNG_SIGNATURE.header_pattern));
    assert!(
        extracted[2]
            .bytes
            .starts_with(JPEG_SIGNATURE.header_pattern)
    );
}

#[test]
fn recover_bytes_end_with_correct_footer() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::for_carver(SUPPORTED_SIGNATURES.iter().collect(), SECTOR_SIZE);

    let extracted = engine.recover(&mut source).unwrap();

    assert!(extracted[0].bytes.ends_with(JPEG_SIGNATURE.footer_pattern));
    assert!(extracted[1].bytes.ends_with(PNG_SIGNATURE.footer_pattern));
    assert!(extracted[2].bytes.ends_with(JPEG_SIGNATURE.footer_pattern));
}

#[test]
fn recover_with_no_signatures_returns_error() {
    let mut source = Cursor::new(build_test_disk());
    let engine = RecoveryEngine::for_carver(vec![], DEFAULT_CHUNK_SIZE);

    let result = engine.recover(&mut source);

    assert!(matches!(result, Err(EngineError::NoSignaturesConfigured)));
}

// ── save_all tests ────────────────────────────────────────────────────────────

fn temp_output_dir(suffix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rust_recovery_out_{}", suffix));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn save_all_creates_output_directory_and_files() {
    let output_dir = temp_output_dir("save_all");
    let engine =
        RecoveryEngine::for_carver(SUPPORTED_SIGNATURES.iter().collect(), DEFAULT_CHUNK_SIZE)
            .with_output_dir(&output_dir);

    let extracted = vec![
        ExtractedFile {
            filename: "recovered_0.jpg".to_string(),
            extension: "jpg".to_string(),
            bytes: JPEG_SIGNATURE.header_pattern.to_vec(),
        },
        ExtractedFile {
            filename: "recovered_1.png".to_string(),
            extension: "png".to_string(),
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
    let engine =
        RecoveryEngine::for_carver(SUPPORTED_SIGNATURES.iter().collect(), DEFAULT_CHUNK_SIZE)
            .with_output_dir(&output_dir);

    let expected_bytes = vec![0x01, 0x02, 0x03, 0x04];
    let extracted = vec![ExtractedFile {
        filename: "recovered_0.jpg".to_string(),
        extension: "jpg".to_string(),
        bytes: expected_bytes.clone(),
    }];

    let saved_paths = engine.save_all(&extracted).unwrap();
    let written = std::fs::read(&saved_paths[0]).unwrap();

    assert_eq!(written, expected_bytes);
}

#[test]
fn save_all_without_output_dir_returns_no_output_dir_error() {
    let engine =
        RecoveryEngine::for_carver(SUPPORTED_SIGNATURES.iter().collect(), DEFAULT_CHUNK_SIZE);
    let extracted = vec![ExtractedFile {
        filename: "recovered_0.jpg".to_string(),
        extension: "jpg".to_string(),
        bytes: vec![0x01],
    }];

    let result = engine.save_all(&extracted);

    assert!(matches!(result, Err(EngineError::NoOutputDir)));
}
