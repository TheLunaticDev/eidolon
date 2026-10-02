use std::fs;

use anyhow::Context;
use rusqlite::{OptionalExtension, params};

use crate::{chunkers::chunker, db::open_connection};

pub fn add_to_db(
    db_path: &std::path::Path,
    source_path: &std::path::PathBuf,
    tags: Option<&Vec<String>>,
    title: Option<&String>,
    author: Option<&String>,
) -> anyhow::Result<()> {
    let contents = fs::read_to_string(source_path)?;
    let content_hash = blake3::hash(contents.as_bytes()).to_string();
    let path_str = source_path.to_string_lossy().to_string();

    let mut conn = open_connection(db_path)?;
    let tx = conn.transaction()?;

    let existing: Option<(i64, String)> = tx
        .query_row(
            "SELECT id, content_hash FROM sources WHERE path = ?1",
            params![path_str],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    if let Some((existing_id, existing_hash)) = existing {
        if existing_hash == content_hash {
            println!("Unchanged, skipping: {path_str}");
            return Ok(());
        }
        println!("Pruning old data & chunks...");
        tx.execute(
            "DELETE FROM chunks_fts WHERE rowid IN (SELECT id FROM chunks WHERE source_id = ?1)",
            params![existing_id],
        )?;
        tx.execute("DELETE FROM sources WHERE id = ?1", params![existing_id])?;
    }

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

    tx.execute(
        "INSERT INTO sources (path, kind, title, author, tags, content_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            path_str,
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
         VALUES (?1, ?2)",
            params![source_id, i as i64,],
        )?;
        let chunk_id = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO chunks_fts (rowid, text)
             VALUES (?1, ?2)",
            params![chunk_id, chunk,],
        )?;
    }

    tx.commit()?;
    println!(
        "Successfully Added {} chunks to the database.",
        chunks.len()
    );
    Ok(())
}
