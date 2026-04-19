use std::io::Cursor;

use crate::carved_file::CarvedFile;
use crate::extractor::Extractor;
use crate::signature::FileKind;

// ── helpers ───────────────────────────────────────────────────────────────────

fn carved(kind: FileKind, start: u64, end: u64) -> CarvedFile {
    CarvedFile {
        kind,
        offset_start: start,
        offset_end: end,
    }
}

fn extract_bytes(source: &[u8], start: u64, end: u64) -> Vec<u8> {
    let mut cursor = Cursor::new(source.to_vec());
    let carved_file = carved(FileKind::Jpeg, start, end);
    let mut output: Vec<u8> = Vec::new();

    Extractor::new()
        .extract(&mut cursor, &carved_file, &mut output)
        .expect("extraction should succeed");

    output
}

// ── basic extraction ──────────────────────────────────────────────────────────

#[test]
fn should_extract_bytes_from_beginning_of_source() {
    let source = vec![0xFF, 0xD8, 0xFF, 0xAA, 0xBB, 0xFF, 0xD9];
    let extracted = extract_bytes(&source, 0, 7);
    assert_eq!(extracted, source);
}

#[test]
fn should_extract_bytes_from_middle_of_source() {
    // File sits between prefix and suffix bytes that must not be included.
    let prefix = vec![0x00, 0x11, 0x22];
    let file_bytes = vec![0xFF, 0xD8, 0xFF, 0xAA, 0xFF, 0xD9];
    let suffix = vec![0x33, 0x44];

    let mut source = prefix.clone();
    source.extend_from_slice(&file_bytes);
    source.extend_from_slice(&suffix);

    let start = prefix.len() as u64;
    let end = start + file_bytes.len() as u64;
    let extracted = extract_bytes(&source, start, end);

    assert_eq!(extracted, file_bytes);
}

#[test]
fn should_return_exact_byte_count() {
    let source = vec![0xFF, 0xD8, 0xFF, 0xAA, 0xBB, 0xFF, 0xD9];
    let mut cursor = Cursor::new(source.clone());
    let carved_file = carved(FileKind::Jpeg, 0, 7);
    let mut output: Vec<u8> = Vec::new();

    let bytes_written = Extractor::new()
        .extract(&mut cursor, &carved_file, &mut output)
        .unwrap();

    assert_eq!(bytes_written, 7);
}

#[test]
fn should_extract_single_byte_file() {
    let source = vec![0x00, 0xAB, 0x00];
    let extracted = extract_bytes(&source, 1, 2);
    assert_eq!(extracted, vec![0xAB]);
}

// ── chunk boundary behaviour ──────────────────────────────────────────────────

#[test]
fn should_extract_correctly_when_file_spans_multiple_chunks() {
    // 20 bytes of file data, extracted with chunk_size = 4 (5 iterations).
    let prefix = vec![0x00u8; 10];
    let file_bytes: Vec<u8> = (0x00..0x14).collect(); // 20 bytes: 0x00..0x13
    let mut source = prefix.clone();
    source.extend_from_slice(&file_bytes);

    let start = prefix.len() as u64;
    let end = start + file_bytes.len() as u64;

    let mut cursor = Cursor::new(source);
    let carved_file = carved(FileKind::Jpeg, start, end);
    let mut output: Vec<u8> = Vec::new();

    Extractor::new()
        .with_chunk_size(4)
        .extract(&mut cursor, &carved_file, &mut output)
        .unwrap();

    assert_eq!(output, file_bytes);
}

#[test]
fn should_extract_correctly_when_chunk_size_exceeds_file_size() {
    let file_bytes = vec![0xAA, 0xBB, 0xCC];
    let source = file_bytes.clone();

    let extracted = extract_bytes(&source, 0, 3);
    assert_eq!(extracted, file_bytes);
}

// ── PNG extraction ────────────────────────────────────────────────────────────

#[test]
fn should_extract_png_bytes_correctly() {
    let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let png_footer = vec![0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82];

    let mut file_bytes = png_header.clone();
    file_bytes.extend_from_slice(&[0x11, 0x22, 0x33]);
    file_bytes.extend_from_slice(&png_footer);

    let prefix = vec![0x00u8; 5];
    let mut source = prefix.clone();
    source.extend_from_slice(&file_bytes);

    let start = prefix.len() as u64;
    let end = start + file_bytes.len() as u64;

    let mut cursor = Cursor::new(source);
    let carved_file = carved(FileKind::Png, start, end);
    let mut output: Vec<u8> = Vec::new();

    Extractor::new()
        .extract(&mut cursor, &carved_file, &mut output)
        .unwrap();

    assert_eq!(output, file_bytes);
}

// ── error cases ───────────────────────────────────────────────────────────────

#[test]
fn should_return_error_for_inverted_range() {
    let source = vec![0xAA, 0xBB, 0xCC];
    let mut cursor = Cursor::new(source);
    let carved_file = carved(FileKind::Jpeg, 5, 2); // end < start
    let mut output: Vec<u8> = Vec::new();

    let result = Extractor::new().extract(&mut cursor, &carved_file, &mut output);

    assert!(result.is_err());
}

#[test]
fn should_return_error_for_zero_length_range() {
    let source = vec![0xAA, 0xBB, 0xCC];
    let mut cursor = Cursor::new(source);
    let carved_file = carved(FileKind::Jpeg, 2, 2); // start == end
    let mut output: Vec<u8> = Vec::new();

    let result = Extractor::new().extract(&mut cursor, &carved_file, &mut output);

    assert!(result.is_err());
}

#[test]
fn should_return_error_when_source_ends_before_offset_end() {
    // Source has only 4 bytes but CarvedFile claims 10.
    let source = vec![0xFF, 0xD8, 0xFF, 0xAA];
    let mut cursor = Cursor::new(source);
    let carved_file = carved(FileKind::Jpeg, 0, 10);
    let mut output: Vec<u8> = Vec::new();

    let result = Extractor::new().extract(&mut cursor, &carved_file, &mut output);

    assert!(result.is_err());
}
