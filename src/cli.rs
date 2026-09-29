use clap::Parser;
use std::path::PathBuf;

use crate::commands::Commands;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long, global = true, default_value = "eidolon.db")]
    pub db: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}


pub fn get_cli() -> Cli {
    Cli::parse()
}
