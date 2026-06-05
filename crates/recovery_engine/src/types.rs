/// Identifies which recovery strategy the engine should use.
///
/// This is the canonical representation shared by all consumers (CLI, UI).
/// Each consumer is responsible for mapping its own input types to this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyKind {
    /// Signature-based file carving — works on any device regardless of filesystem.
    Carver,
    /// FAT32 filesystem-aware recovery — preserves original filenames and sizes.
    Fat32,
}

/// Metadata of a recoverable file produced by a scan-only pass.
///
/// Unlike [`ExtractedFile`], this type carries no byte content — it describes
/// what *can* be recovered without loading file data into memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInfo {
    /// Filename that would be assigned on recovery, e.g. `recovered_0.jpg`.
    pub filename: String,
    /// Lowercase extension without the leading dot, e.g. `"jpg"`.
    pub extension: String,
    /// File size in bytes.
    pub size_bytes: usize,
}

/// A file extracted from a disk source, held in memory.
///
/// Produced by [`crate::strategies::RecoveryStrategy::recover`] and consumed
/// by [`crate::engine::RecoveryEngine::save_all`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedFile {
    /// Filename to use when saving, e.g. `recovered_0.jpg`.
    pub filename: String,
    /// Lowercase file extension without the leading dot, e.g. `"jpg"`, `"txt"`.
    pub extension: String,
    pub bytes: Vec<u8>,
}

fn is_image_ext(ext: &str) -> bool {
    matches!(ext, "jpg" | "jpeg" | "png")
}

impl FileInfo {
    pub fn is_image(&self) -> bool {
        is_image_ext(&self.extension)
    }
}

impl ExtractedFile {
    pub fn is_image(&self) -> bool {
        is_image_ext(&self.extension)
    }
}

/// A combined `Read + Seek` supertrait that makes [`crate::strategies::RecoveryStrategy`]
/// object-safe.
///
/// Rust does not allow `dyn Read + Seek` directly because a trait object can
/// have only one principal (non-auto) trait. Defining this supertrait solves
/// the problem: any `T: Read + Seek` automatically implements `ReadSeek` via
/// the blanket impl below, and `dyn ReadSeek` can be used as a single trait
/// object wherever both `Read` and `Seek` are required.
pub trait ReadSeek: std::io::Read + std::io::Seek {}
impl<T: std::io::Read + std::io::Seek> ReadSeek for T {}
