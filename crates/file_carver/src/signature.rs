pub struct Signature {
    pub name: &'static str,
    pub header_pattern: &'static [u8],
    pub footer_pattern: &'static [u8],
}

pub const JPEG_SIGNATURE: Signature = Signature {
    name: "JPEG",
    header_pattern: &[0xFF, 0xD8, 0xFF],
    footer_pattern: &[0xFF, 0xD9],
};
