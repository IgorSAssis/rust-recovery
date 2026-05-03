use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use super::Command;
use crate::constants::DEFAULT_HEXDUMP_LENGTH;
use crate::validation::SourceValidator;

#[derive(Args)]
pub struct HexdumpArgs {
    #[arg(short, long, help = "Path to the disk image or device to inspect")]
    pub source: PathBuf,

    #[arg(long, default_value_t = 0, help = "Byte offset to start reading from")]
    pub offset: u64,

    #[arg(
        long,
        default_value_t = DEFAULT_HEXDUMP_LENGTH,
        help = "Number of bytes to read"
    )]
    pub length: usize,
}

pub struct HexdumpCommand {
    args: HexdumpArgs,
}

impl HexdumpCommand {
    pub fn new(args: HexdumpArgs) -> Self {
        Self { args }
    }

    fn format_hexdump(buffer: &[u8]) -> String {
        let mut output = String::new();

        for (i, byte) in buffer.iter().enumerate() {
            if i % 16 == 0 {
                output.push_str(&format!("\n{:08x}: ", i));
            }

            output.push_str(&format!("{:02x} ", byte));
        }

        output.push('\n');

        output
    }
}

impl Command for HexdumpCommand {
    fn run(&mut self) -> Result<()> {
        SourceValidator::validate(&self.args.source)?;

        let mut source = File::open(&self.args.source)
            .with_context(|| format!("Cannot open '{}'", self.args.source.display()))?;

        let file_size = source.metadata()?.len();

        if self.args.offset >= file_size {
            anyhow::bail!(
                "Offset {} is beyond the end of '{}' ({} bytes)",
                self.args.offset,
                self.args.source.display(),
                file_size
            );
        }

        let readable = (file_size - self.args.offset) as usize;
        let length = self.args.length.min(readable);

        source
            .seek(SeekFrom::Start(self.args.offset))
            .context("Failed to seek to offset")?;

        let mut buffer = vec![0u8; length];
        source
            .read_exact(&mut buffer)
            .context("Failed to read bytes")?;

        println!(
            "Source: {}  |  Offset: {}  |  Length: {} bytes",
            self.args.source.display(),
            self.args.offset,
            length,
        );

        print!("{}", Self::format_hexdump(&buffer));

        Ok(())
    }
}

#[cfg(test)]
#[path = "hexdump_tests.rs"]
mod hexdump_tests;
