use anyhow::Result;
use clap::Parser;

use cli::app::{Cli, Commands};
use cli::commands::Command;
use cli::commands::hexdump::HexdumpCommand;
use cli::commands::recover::RecoverCommand;
use cli::commands::scan::ScanCommand;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan(args) => ScanCommand::new(args).run(),
        Commands::Recover(args) => RecoverCommand::new(args).run(),
        Commands::Hexdump(args) => HexdumpCommand::new(args).run(),
    }
}
