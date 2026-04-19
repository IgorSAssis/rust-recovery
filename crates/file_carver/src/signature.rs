use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    Jpeg,
    Png,
}

impl FileKind {
    pub fn name(self) -> &'static str {
        match self {
            FileKind::Jpeg => "JPEG",
            FileKind::Png => "PNG",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            FileKind::Jpeg => "jpg",
            FileKind::Png => "png",
        }
    }
}

impl fmt::Display for FileKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

pub struct Signature {
    pub kind: FileKind,
    pub header_pattern: &'static [u8],
    pub footer_pattern: &'static [u8],
}

pub const JPEG_SIGNATURE: Signature = Signature {
    kind: FileKind::Jpeg,
    header_pattern: &[0xFF, 0xD8, 0xFF],
    footer_pattern: &[0xFF, 0xD9],
};

pub const PNG_SIGNATURE: Signature = Signature {
    kind: FileKind::Png,
    header_pattern: &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
    footer_pattern: &[0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82],
};

pub const SUPPORTED_SIGNATURES: &[Signature] = &[JPEG_SIGNATURE, PNG_SIGNATURE];
