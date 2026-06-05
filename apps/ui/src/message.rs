use std::path::PathBuf;

use recovery_engine::types::ExtractedFile;
pub use recovery_engine::types::StrategyKind;

/// All possible actions that can change the application state.
#[derive(Debug, Clone)]
pub enum Message {
    SourcePathChanged(String),
    StrategyChanged(StrategyKind),
    ScanPressed,
    /// Background scan finished — either a list of files or an error string.
    ScanCompleted(Result<Vec<ExtractedFile>, String>),
    FileSelected(usize),
    FileToggled(usize),
    ToggleAll,
    ExportPressed,
    /// Native folder picker returned a path (or `None` if cancelled).
    FolderPicked(Option<PathBuf>),
    /// Export worker finished — carries number of files written.
    ExportCompleted(usize),
    /// Export worker failed — carries human-readable error.
    ExportFailed(String),
    BackToScan,
}

