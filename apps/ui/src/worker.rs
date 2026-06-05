use std::collections::HashSet;
use std::path::PathBuf;

use device_detector::{DeviceDetector, StorageDevice};
use file_carver::constants::DEFAULT_CHUNK_SIZE;
use file_carver::signature::SUPPORTED_SIGNATURES;
use recovery_engine::engine::RecoveryEngine;
use recovery_engine::strategies::Fat32Strategy;
use recovery_engine::types::ExtractedFile;

use crate::message::StrategyKind;

pub struct Worker;

impl Worker {
    /// Runs the recovery engine on a blocking OS thread so the UI stays responsive.
    pub async fn run_scan(
        source_path: String,
        strategy: StrategyKind,
    ) -> Result<Vec<ExtractedFile>, String> {
        tokio::task::spawn_blocking(move || {
            let mut file = std::fs::File::open(&source_path)
                .map_err(|e| format!("Cannot open '{}': {}", source_path, e))?;

            match strategy {
                StrategyKind::Carver => {
                    let signatures: Vec<_> = SUPPORTED_SIGNATURES.iter().collect();
                    RecoveryEngine::for_carver(signatures, DEFAULT_CHUNK_SIZE)
                        .recover(&mut file)
                        .map_err(|e| e.to_string())
                }
                StrategyKind::Fat32 => RecoveryEngine::for_strategy(Box::new(Fat32Strategy))
                    .recover(&mut file)
                    .map_err(|e| e.to_string()),
            }
        })
        .await
        .map_err(|e| e.to_string())?
    }

    /// Lists physical storage devices available on the system.
    pub async fn detect_devices() -> Result<Vec<StorageDevice>, String> {
        tokio::task::spawn_blocking(|| {
            DeviceDetector::new()
                .list_devices()
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())?
    }

    /// Opens the OS native folder-picker dialog and returns the chosen path (or `None` if cancelled).
    ///
    /// Uses the synchronous `rfd::FileDialog` inside `spawn_blocking` to avoid
    /// runtime conflicts with the `ashpd` XDG-portal backend on Linux.
    pub async fn pick_folder(title: &'static str) -> Option<PathBuf> {
        tokio::task::spawn_blocking(move || rfd::FileDialog::new().set_title(title).pick_folder())
            .await
            .unwrap_or(None)
    }

    /// Writes the selected files to `output_dir`, returning the number of files written.
    pub async fn run_export(
        files: Vec<ExtractedFile>,
        selected: HashSet<usize>,
        output_dir: PathBuf,
    ) -> Result<usize, String> {
        tokio::task::spawn_blocking(move || {
            let mut count = 0usize;

            for (i, file) in files.iter().enumerate() {
                if !selected.contains(&i) {
                    continue;
                }
                let dest = output_dir.join(&file.filename);
                std::fs::write(&dest, &file.bytes)
                    .map_err(|e| format!("Failed to write '{}': {}", file.filename, e))?;
                count += 1;
            }

            Ok(count)
        })
        .await
        .map_err(|e| e.to_string())?
    }
}
