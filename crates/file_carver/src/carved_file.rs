use crate::signature::FileKind;

/// A file recovered (or recoverable) from raw disk data.
/// Holds the absolute byte offsets within the source device/image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarvedFile {
    pub kind: FileKind,
    /// Absolute offset of the first byte of the file header in the source.
    pub offset_start: u64,
    /// Absolute offset of the byte immediately after the file footer.
    pub offset_end: u64,
}

impl CarvedFile {
    pub fn size(&self) -> u64 {
        self.offset_end.saturating_sub(self.offset_start)
    }
}
