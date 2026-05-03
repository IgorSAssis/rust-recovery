use file_carver::signature::FileKind;

/// A file extracted from a disk source, held in memory.
///
/// Produced by [`crate::strategies::RecoveryStrategy::recover`] and consumed
/// by [`crate::engine::RecoveryEngine::save_all`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedFile {
    /// Filename to use when saving, e.g. `recovered_0.jpg`.
    pub filename: String,
    pub kind: FileKind,
    pub bytes: Vec<u8>,
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
