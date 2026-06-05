use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, ValueEnum};
use file_carver::constants::DEFAULT_CHUNK_SIZE;
use file_carver::signature::{FileKind, SUPPORTED_SIGNATURES, Signature};
use recovery_engine::engine::RecoveryEngine;
use recovery_engine::strategies::Fat32Strategy;

use recovery_engine::types::StrategyKind;

use super::Command;
use crate::progress::{IndicatifReporter, ProgressReporter};
use crate::validation::SourceValidator;

/// CLI argument adapter for strategy selection.
///
/// Exists solely to satisfy `clap::ValueEnum` — converts to the canonical
/// `StrategyKind` from `recovery_engine` via `From` before use.
#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum StrategyArg {
    /// Signature-based file carver — works on any device regardless of filesystem
    Carver,
    /// FAT32 filesystem-aware recovery — use on FAT32-formatted devices for more
    /// accurate filename and size information
    Fat32,
}

impl From<StrategyArg> for StrategyKind {
    fn from(arg: StrategyArg) -> Self {
        match arg {
            StrategyArg::Carver => StrategyKind::Carver,
            StrategyArg::Fat32 => StrategyKind::Fat32,
        }
    }
}

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

    #[arg(
        long,
        value_enum,
        default_value = "carver",
        help = "Recovery strategy: 'carver' (signature-based, any device) or 'fat32' (filesystem-aware)"
    )]
    pub strategy: StrategyArg,
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
        SourceValidator::validate(&self.args.source)?;

        let mut source = File::open(&self.args.source)
            .with_context(|| format!("Cannot open '{}'", self.args.source.display()))?;

        let file_size = source.metadata()?.len();

        println!("RustRecover — recover");
        println!(
            "Source:  {} ({} bytes)",
            self.args.source.display(),
            file_size
        );
        println!("Output:  {}", self.args.output.display());
        println!("Strategy: {}", self.strategy_label());
        println!();

        match self.args.strategy {
            StrategyArg::Carver => self.run_carver(&mut source),
            StrategyArg::Fat32 => self.run_fat32(&mut source),
        }
    }
}

impl RecoverCommand {
    fn strategy_label(&self) -> &str {
        match self.args.strategy {
            StrategyArg::Carver => "carver (signature-based)",
            StrategyArg::Fat32 => "fat32 (filesystem-aware)",
        }
    }

    fn run_carver(&mut self, source: &mut File) -> Result<()> {
        let type_filter = match &self.args.types {
            None => "all types".to_string(),
            Some(kinds) => kinds
                .iter()
                .map(|kind| kind.extension())
                .collect::<Vec<_>>()
                .join(", "),
        };

        println!("Filter:  {}", type_filter);
        println!();
        println!("Scanning...");

        let signatures = self.resolve_signatures();
        let engine = RecoveryEngine::for_carver(signatures, self.args.chunk_size)
            .with_output_dir(&self.args.output);

        let extracted_files = engine.recover(source).context("Recovery failed")?;

        if extracted_files.is_empty() {
            println!("No recoverable files found.");
            return Ok(());
        }

        println!("Found {} file(s) to recover.", extracted_files.len());
        println!();

        self.save_and_report(&engine, &extracted_files)
    }

    fn run_fat32(&mut self, source: &mut File) -> Result<()> {
        println!("Scanning FAT32 directory entries...");

        let engine = RecoveryEngine::for_strategy(Box::new(Fat32Strategy))
            .with_output_dir(&self.args.output);

        let extracted_files = engine
            .recover(source)
            .context("FAT32 recovery failed — is this a FAT32-formatted device?")?;

        if extracted_files.is_empty() {
            println!("No recoverable files found.");
            return Ok(());
        }

        println!("Found {} deleted file(s).", extracted_files.len());
        println!();

        self.save_and_report(&engine, &extracted_files)
    }

    fn save_and_report(
        &mut self,
        engine: &RecoveryEngine,
        extracted_files: &[recovery_engine::types::ExtractedFile],
    ) -> Result<()> {
        let saved_paths = engine
            .save_all(extracted_files)
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
