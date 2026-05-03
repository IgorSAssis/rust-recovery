use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use cli::app::{Cli, Commands};
use cli::commands::Command;
use cli::commands::devices::DevicesCommand;
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

    let log_level = if cli.verbose { "debug" } else { "warn" };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(log_level))
        .init();

    match cli.command {
        Commands::Scan(args) => ScanCommand::new(args).run(),
        Commands::Recover(args) => RecoverCommand::new(args).run(),
        Commands::Hexdump(args) => HexdumpCommand::new(args).run(),
        Commands::Devices(args) => DevicesCommand::new(args).run(),
    }
}
