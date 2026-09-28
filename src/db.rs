use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;

const SCHEMA_VERSION: i64 = 1;

fn enable_fk(conn: &Connection) -> Result<()> {
    conn.set_db_config(
        rusqlite::config::DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY,
        true,
    )
    .context("Failed to set Enable Foreign Key enforcement.")?;

    Ok(())
}

pub fn create_connection(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path).with_context(|| {
        format!(
            "Failed to create a connection with database \
                     at: {}",
            path.display()
        )
    })?;

    enable_fk(&conn)?;
    Ok(conn)
}

pub fn init_schema(conn: &Connection) -> Result<()> {
    let user_version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if user_version > SCHEMA_VERSION {
        anyhow::bail!("Database schema version {user_version} is newer than this binary supports ({SCHEMA_VERSION})");
    }

    conn.execute_batch(
            "
         CREATE TABLE IF NOT EXISTS sources (
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

    if user_version < SCHEMA_VERSION {
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    }

    Ok(())
}
