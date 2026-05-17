use std::path::PathBuf;

use recovery_engine::types::ExtractedFile;

/// All possible actions that can change the application state.
#[derive(Debug, Clone)]
pub enum Message {
    /// User typed in the source path field.
    SourcePathChanged(String),
    /// User toggled the strategy radio button.
    StrategyChanged(StrategyKind),
    /// User clicked the "Scan" button.
    ScanPressed,
    /// Background scan finished — either a list of files or an error string.
    ScanCompleted(Result<Vec<ExtractedFile>, String>),
    /// User clicked a file row (for preview).
    FileSelected(usize),
    /// User toggled a file's export checkbox.
    FileToggled(usize),
    /// User clicked "Select All" or "Deselect All".
    ToggleAll,
    /// User clicked the "Export Selected" button.
    ExportPressed,
    /// Native folder picker returned a path (or None if cancelled).
    FolderPicked(Option<PathBuf>),
    /// Export worker finished successfully — carries number of files written.
    ExportCompleted(usize),
    /// Export worker failed — carries human-readable error.
    ExportFailed(String),
    /// User clicked "← Back" from the results screen.
    BackToScan,
}

/// Which recovery strategy the user selected.
///
/// `Copy` is required by iced's `radio` widget (`V: Copy + PartialEq`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyKind {
    Carver,
    Fat32,
}

/// State of the export operation.
#[derive(Debug, Clone, PartialEq)]
pub enum ExportState {
    Idle,
    Picking,
    Exporting,
    Done(usize),
    Failed(String),
}
