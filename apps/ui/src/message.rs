use std::path::PathBuf;

use device_detector::StorageDevice;
use recovery_engine::types::ExtractedFile;
pub use recovery_engine::types::StrategyKind;

use crate::screen::Screen;

/// All possible actions that can change the application state.
#[derive(Debug, Clone)]
pub enum Message {
    // ── navigation ────────────────────────────────────────────────────────────
    NavigateTo(Screen),

    // ── devices ───────────────────────────────────────────────────────────────
    DetectDevicesPressed,
    DevicesDetected(Result<Vec<StorageDevice>, String>),
    DeviceSelected(PathBuf),

    // ── scan ──────────────────────────────────────────────────────────────────
    SourcePathChanged(String),
    StrategyChanged(StrategyKind),
    ScanPressed,
    ScanCompleted(Result<Vec<ExtractedFile>, String>),

    // ── results ───────────────────────────────────────────────────────────────
    FileSelected(usize),
    FileToggled(usize),
    ToggleAll,
    ExportPressed,
    FolderPicked(Option<PathBuf>),
    ExportCompleted(usize),
    ExportFailed(String),
}
