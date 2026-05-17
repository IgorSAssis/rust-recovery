use file_carver::signature::SUPPORTED_SIGNATURES;
use recovery_engine::engine::RecoveryEngine;
use recovery_engine::strategies::Fat32Strategy;
use recovery_engine::types::ExtractedFile;

use crate::message::StrategyKind;

/// Runs the recovery engine on a blocking OS thread so the UI stays responsive.
///
/// The result is either the list of extracted files or a human-readable error.
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
