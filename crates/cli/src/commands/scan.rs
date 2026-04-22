use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use file_carver::constants::DEFAULT_CHUNK_SIZE;
use recovery_engine::engine::RecoveryEngine;

use super::Command;

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
    fn run(&self) -> Result<()> {
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

        let engine = RecoveryEngine::new().with_chunk_size(self.args.chunk_size);

        let carved_files = engine.scan(&mut source).context("Scan failed")?;

        if carved_files.is_empty() {
            println!("No recoverable files found.");
            return Ok(());
        }

        let separator = "-".repeat(62);

        println!(
            "{:<4}  {:<6}  {:>18}  {:>14}  {:>10}",
            "#", "Type", "Start (offset)", "End (offset)", "Size"
        );
        println!("{separator}");

        for (index, carved_file) in carved_files.iter().enumerate() {
            println!(
                "{:<4}  {:<6}  {:>18}  {:>14}  {:>10}",
                index,
                carved_file.kind.name(),
                carved_file.offset_start,
                carved_file.offset_end,
                format!("{} B", carved_file.size()),
            );
        }

        println!("{separator}");
        println!("Total: {} file(s) found", carved_files.len());

        Ok(())
    }
}
