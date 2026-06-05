use crate::error::EngineError;
use crate::types::{ExtractedFile, FileInfo, ReadSeek};

pub trait RecoveryStrategy {
    /// Recovers files from `source`, returning their full content in memory.
    fn recover(&self, source: &mut dyn ReadSeek) -> Result<Vec<ExtractedFile>, EngineError>;

    /// Lists recoverable files without loading their byte content.
    ///
    /// The default implementation calls [`recover`] and discards the bytes.
    /// Strategies that can scan more efficiently (e.g. file carver, which only
    /// needs to locate signatures) should override this method.
    fn scan_only(&self, source: &mut dyn ReadSeek) -> Result<Vec<FileInfo>, EngineError> {
        self.recover(source).map(|files| {
            files
                .into_iter()
                .map(|f| FileInfo {
                    filename: f.filename,
                    extension: f.extension,
                    size_bytes: f.bytes.len(),
                })
                .collect()
        })
    }
}
