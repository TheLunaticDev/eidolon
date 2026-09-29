pub mod init;

use std::path::PathBuf;
use clap::{Subcommand, ValueEnum};

#[derive(Subcommand)]
pub enum Commands {
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
pub enum EntryType {
    Source,
    Chunk,
}
