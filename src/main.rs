use std::path::PathBuf;

use anyhow::{Context, Result};

use clap::{Parser, Subcommand, ValueEnum};
use rusqlite::Connection;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, global = true, default_value = "eidolon.db")]
    db: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes Eidolon and database.
    Init,

    /// Add a source file to Eidolon.
    ///
    /// This command parses the file, breaking the file into
    /// sensible chunks and index that into the database.
    Add {
        /// Source file path.
        path: PathBuf,

        /// Tag source file with given tags.
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,

        /// Uses the filename if not specified.
        #[arg(short, long)]
        title: Option<String>,

        /// Sets the Author of the source file.
        #[arg(short, long)]
        author: Option<String>,
    },

    /// Search (lexical) the database to find possible matches.
    Search { query: String },

    /// Show relevant data about a chunk or source file.
    Show { entry_type: EntryType, id: i64 },

    /// Show status of eidolon database.
    Status,

    /// Removes the Source and all associated chunks with the given id
    /// from database.
    Remove { id: i64 },
}

#[derive(Clone, ValueEnum)]
enum EntryType {
    Source,
    Chunk,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            let conn = Connection::open(&cli.db).with_context(|| {
                format!(
                    "Failed to create a connection with database \
                     at: {}",
                    cli.db.display()
                )
            })?;

            conn.set_db_config(
                rusqlite::config::DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY,
                true,
            )
            .context("Failed to set Enable Foreign Key enforcement.")?;

            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS sources (
                         id INTEGER PRIMARY KEY,
                         path TEXT NOT NULL UNIQUE,
                         kind TEXT NOT NULL,
                         title TEXT NOT NULL,
                         author TEXT,
                         tags TEXT,
                         content_hash TEXT NOT NULL,
                         added_at TEXT DEFAULT CURRENT_TIMESTAMP
                     );
                     CREATE TABLE IF NOT EXISTS chunks (
                         id INTEGER PRIMARY KEY,
                         source_id INTEGER NOT NULL REFERENCES
                         sources(id)
                         ON DELETE CASCADE,
                         chunk_index INTEGER NOT NULL,
                         UNIQUE (source_id, chunk_index)
                     );
                     CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts
                         USING fts5(text, tokenize = 'porter');
                     ",
            )?;

            println!("Successfully initialized db at: {}", cli.db.display());
        }
        Commands::Add { .. } => todo!(),
        Commands::Search { .. } => todo!(),
        Commands::Show { .. } => todo!(),
        Commands::Status { .. } => todo!(),
        Commands::Remove { .. } => todo!(),
    }

    Ok(())
}
