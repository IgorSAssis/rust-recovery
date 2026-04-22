use clap::{Parser, Subcommand};

use crate::commands::scan::ScanArgs;

#[derive(Parser)]
#[command(
    name = "rustrecovery",
    about = "File recovery tool for corrupted or formatted storage devices"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Scan a source and list recoverable files")]
    Scan(ScanArgs),
}
