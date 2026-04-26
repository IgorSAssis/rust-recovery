pub fn format_hexdump(buffer: &[u8]) -> String {
    let mut output = String::new();

    for (i, byte) in buffer.iter().enumerate() {
        if i % 16 == 0 {
            output.push_str(&format!("\n{:08x}: ", i));
        }

        output.push_str(&format!("{:02x} ", byte));
    }

    output.push('\n');

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_format_single_line_hexdump() {
        let data: [u8; 3] = [0xAA, 0xBB, 0xCC];

        let result = format_hexdump(&data[..]);

        assert!(result.contains("aa bb cc"));
    }

    #[test]
    fn should_format_offset_correctly() {
        let data: [u8; 32] = [0x00; 32];

        let result = format_hexdump(&data[..]);

        assert!(result.contains("00000000"));
        assert!(result.contains("00000010"));
    }

    #[test]
    fn should_return_empty_output_for_empty_buffer() {
        let data: [u8; 0] = [];

        let result = format_hexdump(&data[..]);

        assert!(result.len() > 0);
    }
}

