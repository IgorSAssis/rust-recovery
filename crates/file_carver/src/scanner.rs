use crate::signature::Signature;

pub fn find_signatures(buffer: &[u8], signature: &Signature) -> Vec<usize> {
    let mut bytes = Vec::new();
    let pattern = signature.header_pattern;
    let pattern_len = pattern.len();

    if pattern_len == 0 {
        return bytes;
    }

    for i in 0..=buffer.len().saturating_sub(pattern_len) {
        if &buffer[i..i + pattern_len] == pattern {
            bytes.push(i);
        }
    }

    bytes
}

pub fn find_header(buffer: &[u8], signature: &Signature) -> Option<usize> {
    let header_pattern = signature.header_pattern;

    if header_pattern.is_empty() {
        return None;
    }

    buffer
        .windows(header_pattern.len())
        .position(|window| window == header_pattern)
}

pub fn find_footer(buffer: &[u8], signature: &Signature, start_offset: usize) -> Option<usize> {
    let footer = signature.footer_pattern;
    let footer_len = footer.len();

    if footer_len == 0 {
        return None;
    }

    for i in start_offset..=buffer.len().saturating_sub(footer_len) {
        if &buffer[i..i + footer_len] == footer {
            return Some(i + footer_len);
        }
    }

    None
}

pub fn find_file_range(buffer: &[u8], signature: &Signature) -> Option<(usize, usize)> {
    if let Some(start) = find_header(buffer, signature) {
        if let Some(end) = find_footer(buffer, signature, start) {
            return Some((start, end));
        }
    }

    return None;
}

pub fn find_all_file_ranges(buffer: &[u8], signature: &Signature) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let starts = find_signatures(buffer, signature);

    for start in starts {
        if let Some(end) = find_footer(buffer, signature, start) {
            ranges.push((start, end));
        }
    }

    ranges
}
