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
    /// User clicked a file row in the results list.
    FileSelected(usize),
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
