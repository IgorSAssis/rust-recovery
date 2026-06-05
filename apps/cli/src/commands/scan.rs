use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use file_carver::constants::DEFAULT_CHUNK_SIZE;
use file_carver::signature::SUPPORTED_SIGNATURES;
use recovery_engine::engine::RecoveryEngine;

use super::Command;
use crate::validation::SourceValidator;

#[derive(Args)]
pub struct ScanArgs {
    #[arg(short, long, help = "Path to the disk image or device to scan")]
    pub source: PathBuf,

    #[arg(
        long,
        default_value_t = DEFAULT_CHUNK_SIZE,
        help = "Number of bytes read per iteration (tune for memory usage)"
    )]
    pub chunk_size: usize,
}

pub struct ScanCommand {
    args: ScanArgs,
}

impl ScanCommand {
    pub fn new(args: ScanArgs) -> Self {
        Self { args }
    }
}

impl Command for ScanCommand {
    fn run(&mut self) -> Result<()> {
        SourceValidator::validate(&self.args.source)?;

        let mut source = File::open(&self.args.source)
            .with_context(|| format!("Cannot open '{}'", self.args.source.display()))?;

        let file_size = source.metadata()?.len();

        println!("RustRecover — scan");
        println!(
            "Source: {} ({} bytes)",
            self.args.source.display(),
            file_size
        );
        println!();

        let engine = RecoveryEngine::for_carver(
            SUPPORTED_SIGNATURES.iter().collect(),
            self.args.chunk_size,
        );

        let file_infos = engine.scan(&mut source).context("Scan failed")?;

        if file_infos.is_empty() {
            println!("No recoverable files found.");
            return Ok(());
        }

        let separator = "-".repeat(50);

        println!("{:<4}  {:<6}  Size", "#", "Type");
        println!("{separator}");

        for (index, file_info) in file_infos.iter().enumerate() {
            println!(
                "{:<4}  {:<6}  {} B",
                index,
                file_info.extension.to_uppercase(),
                file_info.size_bytes,
            );
        }

        println!("{separator}");
        println!("Total: {} file(s) found", file_infos.len());

        Ok(())
    }
}
