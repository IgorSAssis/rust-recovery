use crate::matcher::PatternMatcher;

// ── find_in ───────────────────────────────────────────────────────────────────

#[test]
fn find_in_returns_none_for_empty_pattern() {
    let matcher = PatternMatcher::new(&[]);
    assert_eq!(matcher.find_in(&[0x01, 0x02, 0x03], 0), None);
}

#[test]
fn find_in_returns_none_when_pattern_is_absent() {
    let matcher = PatternMatcher::new(&[0xFF, 0xD8]);
    assert_eq!(matcher.find_in(&[0x00, 0x11, 0x22], 0), None);
}

#[test]
fn find_in_finds_pattern_at_the_start_of_buffer() {
    let matcher = PatternMatcher::new(&[0xFF, 0xD8]);
    assert_eq!(matcher.find_in(&[0xFF, 0xD8, 0x00], 0), Some(0));
}

#[test]
fn find_in_finds_pattern_in_the_middle_of_buffer() {
    let matcher = PatternMatcher::new(&[0xFF, 0xD8]);
    assert_eq!(matcher.find_in(&[0x00, 0x11, 0xFF, 0xD8, 0x22], 0), Some(2));
}

#[test]
fn find_in_finds_pattern_at_the_end_of_buffer() {
    let matcher = PatternMatcher::new(&[0xFF, 0xD8]);
    assert_eq!(matcher.find_in(&[0x00, 0x11, 0xFF, 0xD8], 0), Some(2));
}

#[test]
fn find_in_respects_from_offset_and_skips_earlier_match() {
    // Pattern appears at index 0 and again at index 3.
    // Starting from 1 must skip the first match and return 3.
    let matcher = PatternMatcher::new(&[0xFF, 0xD8]);
    assert_eq!(matcher.find_in(&[0xFF, 0xD8, 0x00, 0xFF, 0xD8], 1), Some(3));
}

#[test]
fn find_in_returns_none_when_from_leaves_no_room_for_match() {
    let matcher = PatternMatcher::new(&[0xFF, 0xD8]);
    // buffer has 3 bytes; last valid start for a 2-byte pattern is index 1.
    // Starting from 2 leaves only 1 byte — no room for a match.
    assert_eq!(matcher.find_in(&[0xFF, 0xD8, 0x00], 2), None);
}

#[test]
fn find_in_works_for_single_byte_pattern() {
    let matcher = PatternMatcher::new(&[0xAA]);
    assert_eq!(matcher.find_in(&[0x00, 0xAA, 0xFF], 0), Some(1));
}

// ── find_all_in ───────────────────────────────────────────────────────────────

#[test]
fn find_all_in_returns_empty_for_empty_pattern() {
    let matcher = PatternMatcher::new(&[]);
    assert!(matcher.find_all_in(&[0x01, 0x02]).is_empty());
}

#[test]
fn find_all_in_returns_empty_when_pattern_is_absent() {
    let matcher = PatternMatcher::new(&[0xFF, 0xD8]);
    assert!(matcher.find_all_in(&[0x00, 0x11, 0x22]).is_empty());
}

#[test]
fn find_all_in_finds_single_occurrence() {
    let matcher = PatternMatcher::new(&[0xFF, 0xD8]);
    assert_eq!(matcher.find_all_in(&[0x00, 0xFF, 0xD8, 0x11]), vec![1]);
}

#[test]
fn find_all_in_finds_multiple_non_overlapping_occurrences() {
    let matcher = PatternMatcher::new(&[0xFF, 0xD8]);
    let buffer = &[0xFF, 0xD8, 0x00, 0xFF, 0xD8, 0x11];
    assert_eq!(matcher.find_all_in(buffer), vec![0, 3]);
}

#[test]
fn find_all_in_finds_overlapping_occurrences() {
    // Pattern [0xAA, 0xAA] overlaps in [0xAA, 0xAA, 0xAA]:
    // valid matches at index 0 and index 1.
    let matcher = PatternMatcher::new(&[0xAA, 0xAA]);
    assert_eq!(matcher.find_all_in(&[0xAA, 0xAA, 0xAA]), vec![0, 1]);
}
