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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_schema_idempotency() -> anyhow::Result<()> {
        let conn = Connection::open_in_memory()?;
        init_schema(&conn)?;
        init_schema(&conn)?;
        Ok(())
    }

    #[test]
    fn test_init_schema_tables_exist() -> anyhow::Result<()> {
        let conn = Connection::open_in_memory()?;
        init_schema(&conn)?;
        assert!(conn.table_exists(None, "sources")?);
        assert!(conn.table_exists(None, "chunks")?);
        assert!(conn.table_exists(None, "chunks_fts")?);
        Ok(())
    }

    #[test]
    fn test_fk_enforcement() -> anyhow::Result<()> {
        let conn = Connection::open_in_memory()?;
        enable_fk(&conn)?;
        init_schema(&conn)?;
        let result = conn.execute(
            "INSERT INTO chunks (id, source_id, chunk_index)
             VALUES (1, 1, 0);",
            []
        );
        match result {
            Err(rusqlite::Error::SqliteFailure(e, _)) => {
                assert_eq!(e.code, rusqlite::ErrorCode::ConstraintViolation);
            }
            other => panic!("expected a constraint violation, got {other:?}"),
        }

        Ok(())
    }

    #[test]
    fn test_init_schema_cascade_from_source_to_chunk_table() -> anyhow::Result<()> {
        let conn = Connection::open_in_memory()?;
        enable_fk(&conn)?;
        init_schema(&conn)?;
        conn.execute(
            "INSERT INTO sources (id, path, kind, title, content_hash)
             VALUES (1, 'somewhere', 'text', 'something', 'hash')",
            []
        )?;
        conn.execute(
            "INSERT INTO chunks (id, source_id, chunk_index)
             VALUES (1, 1, 0)",
            []
        )?;
        assert!(
            conn.query_row(
                "SELECT EXISTS(
                     SELECT * FROM sources WHERE id = 1
                 )",
                [],
                |row| row.get(0)
            )?
        );
        assert!(
            conn.query_row(
                "SELECT EXISTS(
                     SELECT * FROM chunks WHERE id = 1
                 )",
                [],
                |row| row.get(0)
            )?
        );
        conn.execute(
            "DELETE FROM sources
             WHERE id = 1",
            []
        )?;
        assert!(
            !conn.query_row(
                "SELECT EXISTS(
                     SELECT * FROM sources WHERE id = 1
                 )",
                [],
                |row| row.get(0)
            )?
        );
        assert!(
            !conn.query_row(
                "SELECT EXISTS(
                     SELECT * FROM chunks WHERE id = 1
                 )",
                [],
                |row| row.get(0)
            )?
        );
        Ok(())
    }

    #[test]
    fn test_init_schema_correct_user_version() -> anyhow::Result<()> {
        let conn = Connection::open_in_memory()?;
        init_schema(&conn)?;
        let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        assert_eq!(version, SCHEMA_VERSION);
        Ok(())
    }

    #[test]
    fn test_chunk_index_uniqueness_per_source() -> anyhow::Result<()> {
        let conn = Connection::open_in_memory()?;
        enable_fk(&conn)?;
        init_schema(&conn)?;
        conn.execute(
            "INSERT INTO sources (id, path, kind, title, content_hash)
             VALUES (1, 'somewhere', 'text', 'something', 'hash')",
            [],
        )?;
        conn.execute(
            "INSERT INTO chunks (id, source_id, chunk_index) VALUES (1, 1, 0)",
            [],
        )?;
        let result = conn.execute(
            "INSERT INTO chunks (id, source_id, chunk_index) VALUES (2, 1, 0)",
            [],
        );
        match result {
            Err(rusqlite::Error::SqliteFailure(e, _)) => {
                assert_eq!(e.code, rusqlite::ErrorCode::ConstraintViolation);
            }
            other => panic!("expected a constraint violation, got {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn test_chunks_fts_porter_stemming() -> anyhow::Result<()> {
        let conn = Connection::open_in_memory()?;
        init_schema(&conn)?;
        conn.execute(
            "INSERT INTO chunks_fts (rowid, text) VALUES (1, 'the generators yield values')",
            [],
        )?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM chunks_fts WHERE chunks_fts MATCH 'generator'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(count, 1);
        Ok(())
    }
}
