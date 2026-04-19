use std::io::Cursor;

use crate::window::ScanWindow;

// ── absolute_offset ───────────────────────────────────────────────────────────

#[test]
fn absolute_offset_is_zero_for_first_window_with_no_overlap() {
    let mut window = ScanWindow::new(0, 4);
    let mut cursor = Cursor::new(vec![0x01, 0x02, 0x03, 0x04]);

    let slice = window.next(&mut cursor).unwrap().unwrap();

    assert_eq!(slice.absolute_offset(0), 0);
    assert_eq!(slice.absolute_offset(3), 3);
}

#[test]
fn absolute_offset_is_correct_for_second_window() {
    // chunk_size=4, overlap_size=2, source has 8 bytes:
    //   window 1: bytes=[01,02,03,04], base=0, overlap=[03,04]
    //   window 2: bytes=[03,04,05,06,07,08], base = 4 - 2 = 2
    //     → absolute_offset(0) = 2  (0x03 lives at disk offset 2)
    //     → absolute_offset(2) = 4  (0x05 lives at disk offset 4)
    let mut window = ScanWindow::new(2, 4);
    let mut cursor = Cursor::new(vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);

    let _ = window.next(&mut cursor).unwrap();
    let slice = window.next(&mut cursor).unwrap().unwrap();

    assert_eq!(slice.absolute_offset(0), 2);
    assert_eq!(slice.absolute_offset(2), 4);
    assert_eq!(slice.absolute_offset(4), 6);
}

// ── ScanWindow ────────────────────────────────────────────────────────────────

#[test]
fn scan_window_returns_none_for_empty_source() {
    let mut window = ScanWindow::new(2, 4);
    let mut cursor = Cursor::new(vec![]);

    assert!(window.next(&mut cursor).unwrap().is_none());
}

#[test]
fn scan_window_first_window_contains_full_first_chunk() {
    let mut window = ScanWindow::new(2, 4);
    let mut cursor = Cursor::new(vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);

    let slice = window.next(&mut cursor).unwrap().unwrap();

    // First window: no prior overlap, so bytes = exactly the first 4 bytes.
    assert_eq!(slice.bytes(), &[0x01, 0x02, 0x03, 0x04]);
    assert_eq!(slice.absolute_offset(0), 0);
}

#[test]
fn scan_window_second_window_starts_with_overlap_from_first_chunk() {
    // overlap_size=2: last 2 bytes of chunk 1 are prepended to window 2.
    //   chunk 1: [01,02,03,04] → overlap=[03,04]
    //   window 2: [03,04,05,06,07,08]
    let mut window = ScanWindow::new(2, 4);
    let mut cursor = Cursor::new(vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);

    let _ = window.next(&mut cursor).unwrap();
    let slice = window.next(&mut cursor).unwrap().unwrap();

    assert_eq!(slice.bytes(), &[0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
}

#[test]
fn scan_window_base_of_second_window_accounts_for_overlap() {
    // base = disk_pos_after_chunk1 - overlap_size = 4 - 2 = 2
    // window2.bytes[0] (0x03) sits at disk offset 2.
    let mut window = ScanWindow::new(2, 4);
    let mut cursor = Cursor::new(vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);

    let _ = window.next(&mut cursor).unwrap();
    let slice = window.next(&mut cursor).unwrap().unwrap();

    assert_eq!(slice.absolute_offset(0), 2);
    assert_eq!(slice.absolute_offset(1), 3);
}

#[test]
fn scan_window_returns_none_after_source_is_exhausted() {
    let mut window = ScanWindow::new(0, 4);
    let mut cursor = Cursor::new(vec![0x01, 0x02]);

    let _ = window.next(&mut cursor).unwrap().unwrap();
    assert!(window.next(&mut cursor).unwrap().is_none());
}

#[test]
fn scan_window_overlap_bridges_pattern_across_chunk_boundary() {
    // A 2-byte pattern [0xAA, 0xBB] split across chunks:
    //   chunk 1: [00, 00, 00, 0xAA] → overlap=[0xAA]
    //   chunk 2: [0xBB, 00, 00, 00]
    //   window 2: [0xAA, 0xBB, 00, 00, 00]  ← pattern is visible here
    //
    // Without overlap, 0xAA and 0xBB would never appear in the same window.
    let mut window = ScanWindow::new(1, 4);
    let mut cursor = Cursor::new(vec![0x00, 0x00, 0x00, 0xAA, 0xBB, 0x00, 0x00, 0x00]);

    let _ = window.next(&mut cursor).unwrap();
    let slice = window.next(&mut cursor).unwrap().unwrap();

    assert_eq!(&slice.bytes()[0..2], &[0xAA, 0xBB]);
    // 0xAA is at local index 0 → absolute offset 3 (end of chunk 1).
    assert_eq!(slice.absolute_offset(0), 3);
}

#[test]
fn scan_window_handles_source_smaller_than_chunk_size() {
    // Source has 3 bytes but chunk_size is 8; partial read must still work.
    let mut window = ScanWindow::new(0, 8);
    let mut cursor = Cursor::new(vec![0xAA, 0xBB, 0xCC]);

    let slice = window.next(&mut cursor).unwrap().unwrap();

    assert_eq!(slice.bytes(), &[0xAA, 0xBB, 0xCC]);
    assert_eq!(slice.absolute_offset(2), 2);
}
