use anyhow::Result;
use clap::Parser;

use cli::app::{Cli, Commands};
use cli::commands::Command;
use cli::commands::recover::RecoverCommand;
use cli::commands::scan::ScanCommand;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan(args) => ScanCommand::new(args).run(),
        Commands::Recover(args) => RecoverCommand::new(args).run(),
    }
}
