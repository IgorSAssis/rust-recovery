use crate::{
    scanner::{find_all_file_ranges, find_file_range, find_footer, find_header},
    signature::PNG_SIGNATURE,
};

use super::signature::JPEG_SIGNATURE;

#[test]
fn should_find_single_jpeg_header() {
    let buffer: [u8; 6] = [0x00, 0x11, 0xFF, 0xD8, 0xFF, 0x22];

    let result = find_header(&buffer, &JPEG_SIGNATURE);

    assert_eq!(result, Some(2));
}

#[test]
fn should_find_jpeg_footer() {
    let buffer: [u8; 6] = [0xFF, 0xD8, 0xFF, 0x11, 0xFF, 0xD9];

    let start = find_header(&buffer, &JPEG_SIGNATURE).unwrap();

    let end = find_footer(&buffer, &JPEG_SIGNATURE, start);

    assert_eq!(end, Some(6));
}

#[test]
fn should_find_complete_jpeg_range() {
    let buffer: [u8; 13] = [
        0x00, 0x11, 0x22, 0xFF, 0xD8, 0xFF, 0xAA, 0xBB, 0xCC, 0xFF, 0xD9, 0x33, 0x44,
    ];

    let range = find_file_range(&buffer, &JPEG_SIGNATURE);

    assert_eq!(range, Some((3, 11)));
}

#[test]
fn should_find_multiple_jpeg_ranges() {
    let buffer: [u8; 21] = [
        0xFF, 0xD8, 0xFF, 0xAA, 0xFF, 0xD9, 0x11, 0x22, 0xFF, 0xD8, 0xFF, 0xBB, 0xFF, 0xD9, 0x33,
        0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
    ];

    let ranges = find_all_file_ranges(&buffer, &JPEG_SIGNATURE);

    assert_eq!(ranges, vec![(0, 6), (8, 14)]);
}

#[test]
fn should_find_single_png_header() {
    let buffer: [u8; 10] = [0x00, 0x00, 0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    let result = find_header(&buffer, &PNG_SIGNATURE);

    assert_eq!(result, Some(2));
}

#[test]
fn should_find_single_png_range() {
    let buffer = [
        0x00, 0x00, 0x00, 0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x11, 0x12, 0x13, 0x14,
        0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let ranges = find_file_range(&buffer, &PNG_SIGNATURE);

    assert_eq!(ranges, Some((3, 23)));
}
