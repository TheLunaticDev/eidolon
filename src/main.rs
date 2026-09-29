mod db;
mod cli;
mod commands;

use commands::{Commands, init::initialize_app};
use anyhow::Result;

fn main() -> Result<()> {
    let cli = cli::get_cli();

    match &cli.command {
        Commands::Init => initialize_app(&cli)?,
        Commands::Add { .. } => todo!(),
        Commands::Search { .. } => todo!(),
        Commands::Show { .. } => todo!(),
        Commands::Status => todo!(),
        Commands::Remove { .. } => todo!(),
    }

    Ok(())
}
