use std::collections::HashSet;
use std::path::PathBuf;

use file_carver::signature::SUPPORTED_SIGNATURES;
use recovery_engine::engine::RecoveryEngine;
use recovery_engine::strategies::Fat32Strategy;
use recovery_engine::types::ExtractedFile;

use crate::message::StrategyKind;

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
                RecoveryEngine::new()
                    .with_signatures(signatures)
                    .recover(&mut file)
                    .map_err(|e| e.to_string())
            }
            StrategyKind::Fat32 => RecoveryEngine::new()
                .with_strategy(Box::new(Fat32Strategy))
                .recover(&mut file)
                .map_err(|e| e.to_string()),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Opens the OS native folder-picker dialog and returns the chosen path (or None if cancelled).
///
/// Uses the synchronous `rfd::FileDialog` inside `spawn_blocking` to avoid
/// async-runtime conflicts with the `ashpd` XDG-portal backend on Linux.
pub async fn pick_folder() -> Option<PathBuf> {
    tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Select output folder for recovered files")
            .pick_folder()
    })
    .await
    .unwrap_or(None)
}

/// Writes selected files to `output_dir`, returning the number of files written.
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

