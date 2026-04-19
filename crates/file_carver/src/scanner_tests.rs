use std::io::Cursor;

use crate::carved_file::CarvedFile;
use crate::scanner::Scanner;
use crate::signature::{FileKind, JPEG_SIGNATURE, PNG_SIGNATURE};

// ── helpers ──────────────────────────────────────────────────────────────────

fn jpeg(start: u64, end: u64) -> CarvedFile {
    CarvedFile {
        kind: FileKind::Jpeg,
        offset_start: start,
        offset_end: end,
    }
}

fn png(start: u64, end: u64) -> CarvedFile {
    CarvedFile {
        kind: FileKind::Png,
        offset_start: start,
        offset_end: end,
    }
}

// ── basic cases ───────────────────────────────────────────────────────────────

#[test]
fn should_return_empty_for_empty_source() {
    let mut cursor = Cursor::new(vec![]);
    let result = Scanner::new()
        .add_signature(&JPEG_SIGNATURE)
        .with_chunk_size(64)
        .scan(&mut cursor)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn should_return_empty_when_no_signatures_match() {
    let data = vec![0xAA, 0xBB, 0xCC, 0xDD];
    let mut cursor = Cursor::new(data);
    let result = Scanner::new()
        .add_signature(&JPEG_SIGNATURE)
        .with_chunk_size(64)
        .scan(&mut cursor)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn should_find_jpeg_entirely_within_single_chunk() {
    // Header FF D8 FF — data — footer FF D9, all within one chunk.
    let data = vec![0xFF, 0xD8, 0xFF, 0xAA, 0xBB, 0xFF, 0xD9, 0x00];
    let mut cursor = Cursor::new(data);
    let result = Scanner::new()
        .add_signature(&JPEG_SIGNATURE)
        .with_chunk_size(64)
        .scan(&mut cursor)
        .unwrap();
    assert_eq!(result, vec![jpeg(0, 7)]);
}

#[test]
fn should_find_jpeg_with_offset_prefix() {
    // File does not start at byte 0.
    let data = vec![0x00, 0x11, 0x22, 0xFF, 0xD8, 0xFF, 0xCC, 0xFF, 0xD9, 0x33];
    let mut cursor = Cursor::new(data);
    let result = Scanner::new()
        .add_signature(&JPEG_SIGNATURE)
        .with_chunk_size(64)
        .scan(&mut cursor)
        .unwrap();
    assert_eq!(result, vec![jpeg(3, 9)]);
}

// ── chunk-boundary cases ──────────────────────────────────────────────────────

#[test]
fn should_find_jpeg_footer_spanning_two_chunks() {
    // chunk_size = 4
    // chunk 1: [FF D8 FF AA]  — header present, footer absent
    // chunk 2: [BB CC FF D9]  — footer present
    let data = vec![0xFF, 0xD8, 0xFF, 0xAA, 0xBB, 0xCC, 0xFF, 0xD9];
    let mut cursor = Cursor::new(data);
    let result = Scanner::new()
        .add_signature(&JPEG_SIGNATURE)
        .with_chunk_size(4)
        .scan(&mut cursor)
        .unwrap();
    assert_eq!(result, vec![jpeg(0, 8)]);
}

#[test]
fn should_find_header_split_across_chunk_boundary() {
    // chunk_size = 16
    // The JPEG header [FF D8 FF] is split: FF sits in chunk 1, D8 FF in chunk 2.
    //
    //   bytes 0-14:  0x00 padding
    //   byte  15:    0xFF  ← first byte of JPEG header (end of chunk 1)
    //   bytes 16-17: 0xD8 0xFF ← rest of header (start of chunk 2)
    //   bytes 18-19: 0xAA 0xBB ← file data
    //   bytes 20-21: 0xFF 0xD9 ← footer
    let mut data = vec![0x00u8; 15];
    data.extend_from_slice(&[0xFF, 0xD8, 0xFF, 0xAA, 0xBB, 0xFF, 0xD9]);

    let mut cursor = Cursor::new(data);
    let result = Scanner::new()
        .add_signature(&JPEG_SIGNATURE)
        .with_chunk_size(16)
        .scan(&mut cursor)
        .unwrap();
    assert_eq!(result, vec![jpeg(15, 22)]);
}

#[test]
fn should_find_footer_split_across_chunk_boundary() {
    // chunk_size = 8
    // The JPEG footer [FF D9] is split: FF at end of chunk 1, D9 at start of chunk 2.
    //
    //   chunk 1: [FF D8 FF AA BB CC DD FF]  — header + data + first footer byte
    //   chunk 2: [D9 00 00 00 00 00 00 00]  — second footer byte
    let data = vec![
        0xFF, 0xD8, 0xFF, 0xAA, 0xBB, 0xCC, 0xDD, 0xFF, // chunk 1
        0xD9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // chunk 2
    ];
    let mut cursor = Cursor::new(data);
    let result = Scanner::new()
        .add_signature(&JPEG_SIGNATURE)
        .with_chunk_size(8)
        .scan(&mut cursor)
        .unwrap();
    assert_eq!(result, vec![jpeg(0, 9)]);
}

// ── multi-file cases ──────────────────────────────────────────────────────────

#[test]
fn should_find_multiple_jpegs_in_sequence() {
    // Two non-overlapping JPEGs separated by random bytes.
    let data = vec![
        // JPEG 1: bytes 0-5
        0xFF, 0xD8, 0xFF, 0xAA, 0xFF, 0xD9, // gap
        0x11, 0x22, // JPEG 2: bytes 8-13
        0xFF, 0xD8, 0xFF, 0xBB, 0xFF, 0xD9,
    ];
    let mut cursor = Cursor::new(data);
    let result = Scanner::new()
        .add_signature(&JPEG_SIGNATURE)
        .with_chunk_size(4)
        .scan(&mut cursor)
        .unwrap();
    assert_eq!(result, vec![jpeg(0, 6), jpeg(8, 14)]);
}

#[test]
fn should_find_jpeg_and_png_independently() {
    // A JPEG followed by a PNG in the same byte stream.
    let data = vec![
        // JPEG: bytes 0-5
        0xFF, 0xD8, 0xFF, 0xCC, 0xFF, 0xD9, // gap
        0x00, 0x00, // PNG header: bytes 8-15
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG body
        0x11, 0x22, 0x33, 0x44, // PNG footer: bytes 20-27
        0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let mut cursor = Cursor::new(data);
    let result = Scanner::new()
        .add_signature(&JPEG_SIGNATURE)
        .add_signature(&PNG_SIGNATURE)
        .with_chunk_size(16)
        .scan(&mut cursor)
        .unwrap();
    assert_eq!(result, vec![jpeg(0, 6), png(8, 28)]);
}

// ── no-duplicate guarantee ────────────────────────────────────────────────────

#[test]
fn should_not_duplicate_file_found_near_chunk_boundary() {
    // chunk_size = 4
    // The JPEG header sits exactly at the boundary so it appears in the
    // overlap of the next window. The scanner must produce exactly one entry.
    //
    //   chunk 1: [AA BB FF D8]         — header starts at byte 2 (split)
    //   chunk 2: [FF CC DD FF D9 EE]   — header completes, footer present
    let data = vec![
        0xAA, 0xBB, 0xFF, 0xD8, // chunk 1
        0xFF, 0xCC, 0xDD, 0xFF, 0xD9, 0xEE, // chunk 2
    ];
    let mut cursor = Cursor::new(data);
    let result = Scanner::new()
        .add_signature(&JPEG_SIGNATURE)
        .with_chunk_size(4)
        .scan(&mut cursor)
        .unwrap();
    assert_eq!(
        result.len(),
        1,
        "expected exactly one file, got: {:?}",
        result
    );
    assert_eq!(result[0].offset_start, 2);
    assert_eq!(result[0].offset_end, 9);
}
