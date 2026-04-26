use super::HexdumpCommand;

#[test]
fn should_format_single_line_hexdump() {
    let data: [u8; 3] = [0xAA, 0xBB, 0xCC];

    let result = HexdumpCommand::format_hexdump(&data);

    assert!(result.contains("aa bb cc"));
}

#[test]
fn should_format_offset_correctly() {
    let data: [u8; 32] = [0x00; 32];

    let result = HexdumpCommand::format_hexdump(&data);

    assert!(result.contains("00000000"));
    assert!(result.contains("00000010"));
}

#[test]
fn should_return_newline_for_empty_buffer() {
    let data: [u8; 0] = [];

    let result = HexdumpCommand::format_hexdump(&data);

    assert_eq!(result, "\n");
}

#[test]
fn should_break_line_every_16_bytes() {
    let data = [0xFFu8; 32];

    let result = HexdumpCommand::format_hexdump(&data);
    let lines: Vec<&str> = result.lines().collect();

    // Each group of 16 bytes starts a new line with the offset address
    assert!(lines.iter().any(|l| l.starts_with("00000000")));
    assert!(lines.iter().any(|l| l.starts_with("00000010")));
}

#[test]
fn should_format_bytes_as_lowercase_hex() {
    let data: [u8; 1] = [0xAB];

    let result = HexdumpCommand::format_hexdump(&data);

    assert!(result.contains("ab"));
    assert!(!result.contains("AB"));
}
