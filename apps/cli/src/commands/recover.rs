use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use file_carver::constants::DEFAULT_CHUNK_SIZE;
use file_carver::signature::{FileKind, Signature, SUPPORTED_SIGNATURES};
use recovery_engine::engine::RecoveryEngine;

use super::Command;
use crate::progress::{IndicatifReporter, ProgressReporter};

#[derive(Args)]
pub struct RecoverArgs {
    #[arg(short, long, help = "Path to the disk image or device to recover from")]
    pub source: PathBuf,

    #[arg(short, long, help = "Directory where recovered files will be saved")]
    pub output: PathBuf,

    #[arg(
        long,
        value_delimiter = ',',
        help = "Comma-separated file types to recover (e.g. jpg,png,pdf). Recovers all types if omitted."
    )]
    pub types: Option<Vec<FileKind>>,

    #[arg(
        long,
        default_value_t = DEFAULT_CHUNK_SIZE,
        help = "Number of bytes read per iteration (tune for memory usage)"
    )]
    pub chunk_size: usize,
}

pub struct RecoverCommand {
    args: RecoverArgs,
    reporter: Box<dyn ProgressReporter>,
}

impl RecoverCommand {
    pub fn new(args: RecoverArgs) -> Self {
        Self {
            args,
            reporter: Box::new(IndicatifReporter::new()),
        }
    }

    pub fn with_reporter(mut self, reporter: Box<dyn ProgressReporter>) -> Self {
        self.reporter = reporter;
        self
    }

    fn resolve_signatures(&self) -> Vec<&'static Signature> {
        match &self.args.types {
            None => SUPPORTED_SIGNATURES.iter().collect(),
            Some(kinds) => SUPPORTED_SIGNATURES
                .iter()
                .filter(|sig| kinds.contains(&sig.kind))
                .collect(),
        }
    }
}

impl Command for RecoverCommand {
    fn run(&mut self) -> Result<()> {
        let mut source = File::open(&self.args.source)
            .with_context(|| format!("Cannot open '{}'", self.args.source.display()))?;

        let file_size = source.metadata()?.len();
        let type_filter = match &self.args.types {
            None => "all types".to_string(),
            Some(kinds) => kinds
                .iter()
                .map(|kind| kind.extension())
                .collect::<Vec<_>>()
                .join(", "),
        };

        println!("RustRecover — recover");
        println!("Source:  {} ({} bytes)", self.args.source.display(), file_size);
        println!("Output:  {}", self.args.output.display());
        println!("Filter:  {}", type_filter);
        println!();
        println!("Scanning...");

        let signatures = self.resolve_signatures();
        let engine = RecoveryEngine::new()
            .with_output_dir(&self.args.output)
            .with_signatures(signatures)
            .with_chunk_size(self.args.chunk_size);

        let carved_files = engine.scan(&mut source).context("Scan failed")?;

        if carved_files.is_empty() {
            println!("No recoverable files found.");
            return Ok(());
        }

        println!("Found {} file(s) to recover.", carved_files.len());
        println!();

        let extracted_files = engine
            .extract_all(&mut source, &carved_files)
            .context("Extraction failed")?;

        let saved_paths = engine
            .save_all(&extracted_files)
            .context("Failed to save files")?;

        debug_assert_eq!(
            extracted_files.len(),
            saved_paths.len(),
            "extract_all and save_all must return the same number of entries"
        );

        self.reporter.set_length(extracted_files.len() as u64);

        for (extracted_file, _path) in extracted_files.iter().zip(saved_paths.iter()) {
            self.reporter
                .set_message(&format!("Saved {}", extracted_file.filename));
            self.reporter.inc(1);
        }

        self.reporter.finish_with_message("Done");

        println!();
        println!(
            "Recovered {} file(s) to '{}':",
            saved_paths.len(),
            self.args.output.display()
        );

        for (extracted_file, path) in extracted_files.iter().zip(saved_paths.iter()) {
            println!(
                "  {}  ({} B)",
                path.file_name().unwrap().to_string_lossy(),
                extracted_file.bytes.len(),
            );
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "recover_tests.rs"]
mod recover_tests;

