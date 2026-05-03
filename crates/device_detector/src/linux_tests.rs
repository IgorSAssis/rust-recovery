use super::LinuxDeviceDetector;

// --- parse_size_bytes ---

#[test]
fn parse_size_bytes_converts_blocks_to_bytes() {
    // 1953525168 blocks × 512 bytes = 1_000_204_886_016 bytes (≈ 931 GiB — typical HDD)
    let result = LinuxDeviceDetector::parse_size_bytes("1953525168\n").unwrap();
    assert_eq!(result, 1953525168 * 512);
}

#[test]
fn parse_size_bytes_handles_zero() {
    let result = LinuxDeviceDetector::parse_size_bytes("0\n").unwrap();
    assert_eq!(result, 0);
}

#[test]
fn parse_size_bytes_trims_whitespace() {
    let result = LinuxDeviceDetector::parse_size_bytes("  2048  \n").unwrap();
    assert_eq!(result, 2048 * 512);
}

#[test]
fn parse_size_bytes_returns_error_for_non_numeric_input() {
    let result = LinuxDeviceDetector::parse_size_bytes("not_a_number");
    assert!(result.is_err());
}

#[test]
fn parse_size_bytes_returns_error_for_empty_string() {
    let result = LinuxDeviceDetector::parse_size_bytes("");
    assert!(result.is_err());
}

// --- parse_removable ---

#[test]
fn parse_removable_returns_true_for_one() {
    assert!(LinuxDeviceDetector::parse_removable("1\n"));
}

#[test]
fn parse_removable_returns_false_for_zero() {
    assert!(!LinuxDeviceDetector::parse_removable("0\n"));
}

#[test]
fn parse_removable_returns_false_for_unexpected_value() {
    assert!(!LinuxDeviceDetector::parse_removable("2\n"));
}

#[test]
fn parse_removable_trims_whitespace() {
    assert!(LinuxDeviceDetector::parse_removable("  1  \n"));
}

// --- parse_model ---

#[test]
fn parse_model_trims_trailing_spaces() {
    // The kernel pads model names to a fixed width with trailing spaces
    let result = LinuxDeviceDetector::parse_model("TOSHIBA HDWD110       \n");
    assert_eq!(result, "TOSHIBA HDWD110");
}

#[test]
fn parse_model_trims_leading_and_trailing_whitespace() {
    let result = LinuxDeviceDetector::parse_model("  Samsung SSD 870  \n");
    assert_eq!(result, "Samsung SSD 870");
}

#[test]
fn parse_model_returns_empty_string_for_blank_content() {
    let result = LinuxDeviceDetector::parse_model("   \n");
    assert_eq!(result, "");
}
