mod db;
mod cli;
mod commands;
mod chunkers;

use std::fs;

use chunkers::chunker;
use commands::{Commands, init::initialize_app};
use anyhow::{Context, Result};
use db::open_connection;
use rusqlite::params;
use blake3;

fn main() -> Result<()> {
    let cli = cli::get_cli();

    match &cli.command {
        Commands::Init => initialize_app(&cli)?,
        Commands::Add { path, tags, title, author } => add_to_db(&cli.db, path, tags.as_ref(), title.as_ref(), author.as_ref())?,
        Commands::Search { .. } => todo!(),
        Commands::Show { .. } => todo!(),
        Commands::Status => todo!(),
        Commands::Remove { .. } => todo!(),
    }

    Ok(())
}

fn add_to_db(db_path: &std::path::PathBuf, source_path: &std::path::PathBuf, tags: Option<&Vec<String>>, title: Option<&String>, author: Option<&String>) -> anyhow::Result<()> {
    let contents = fs::read_to_string(source_path)?;
    let content_hash = blake3::hash(contents.as_bytes()).to_string();
    let chunks = chunker(&contents);

    let title = match title {
        Some(val) => val.clone(),
        None => source_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
            .context("source path has no valid filename")?,
    };
    let tags: Option<String> = tags.map(|v| v.join(","));

    let mut conn = open_connection(db_path)?;
    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO sources (path, kind, title, author, tags, content_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![
             source_path.to_string_lossy().to_string(),
             "text".to_string(),
             title,
             author,
             tags,
             content_hash,
         ],
    )?;

    let source_id = tx.last_insert_rowid();

    for (i, chunk) in chunks.iter().enumerate() {
        tx.execute(
        "INSERT INTO chunks (source_id, chunk_index)
         VALUES (?1, ?2)", params![
             source_id,
             i as i64,
         ])?;
        let chunk_id = tx.last_insert_rowid();

        tx.execute(
            "INSERT INTO chunks_fts (rowid, text)
             VALUES (?1, ?2)", params![
                 chunk_id,
                 chunk,
             ])?;
    }
    
    tx.commit()?;
    println!("Successfully Added {} chunks to the database.", chunks.len());
    Ok(())
}
