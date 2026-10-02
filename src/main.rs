mod chunkers;
mod cli;
mod commands;
mod db;

use anyhow::Result;
use commands::{Commands, add::add_to_db, init::initialize_app};

fn main() -> Result<()> {
    let cli = cli::get_cli();

    match &cli.command {
        Commands::Init => initialize_app(&cli)?,
        Commands::Add {
            path,
            tags,
            title,
            author,
        } => add_to_db(
            &cli.db,
            path,
            tags.as_ref(),
            title.as_ref(),
            author.as_ref(),
        )?,
        Commands::Search { .. } => todo!(),
        Commands::Show { .. } => todo!(),
        Commands::Status => todo!(),
        Commands::Remove { .. } => todo!(),
    }

    Ok(())
}
