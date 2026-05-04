//! SQLite database schema and connection management

use rusqlite::{Connection, Result as SqliteResult, params};
use std::path::Path;

/// Initialize database schema
pub fn init_schema(conn: &Connection) -> SqliteResult<()> {
    // Documents table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            author TEXT,
            date TEXT,
            source_collection TEXT NOT NULL,
            source_url TEXT,
            document_type TEXT NOT NULL,
            text TEXT NOT NULL,
            metadata TEXT,
            ingested_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP
        )",
        [],
    )?;

    // Chunks table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chunks (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id),
            title TEXT,
            text TEXT NOT NULL,
            word_count INTEGER NOT NULL,
            preview TEXT,
            issue_tags TEXT,
            clause_tags TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(document_id) REFERENCES documents(id)
        )",
        [],
    )?;

    // Full-text index table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS fulltext_index (
            term TEXT NOT NULL,
            chunk_id TEXT NOT NULL,
            frequency INTEGER NOT NULL,
            PRIMARY KEY (term, chunk_id),
            FOREIGN KEY(chunk_id) REFERENCES chunks(id)
        )",
        [],
    )?;

    // Fuzzy index tokens table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS fuzzy_tokens (
            token TEXT NOT NULL,
            chunk_id TEXT NOT NULL,
            PRIMARY KEY (token, chunk_id),
            FOREIGN KEY(chunk_id) REFERENCES chunks(id)
        )",
        [],
    )?;

    // Embeddings table (for Phase 2D: Semantic Search)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS embeddings (
            chunk_id TEXT PRIMARY KEY,
            embedding BLOB NOT NULL,
            model_name TEXT,
            computed_at TIMESTAMP,
            FOREIGN KEY(chunk_id) REFERENCES chunks(id)
        )",
        [],
    )?;

    // Create indexes for performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chunks_document_id ON chunks(document_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_fulltext_term ON fulltext_index(term)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_fuzzy_token ON fuzzy_tokens(token)",
        [],
    )?;

    Ok(())
}

/// Open or create database connection
pub fn open_db<P: AsRef<Path>>(path: P) -> SqliteResult<Connection> {
    let conn = Connection::open(path)?;

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Initialize schema
    init_schema(&conn)?;

    Ok(conn)
}

/// Save document to database
pub fn save_document(
    conn: &Connection,
    doc_id: &str,
    title: &str,
    author: Option<&str>,
    date: Option<&str>,
    source_collection: &str,
    source_url: Option<&str>,
    document_type: &str,
    text: &str,
) -> SqliteResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO documents
         (id, title, author, date, source_collection, source_url, document_type, text)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![doc_id, title, author, date, source_collection, source_url, document_type, text],
    )?;
    Ok(())
}

/// Save chunk to database
pub fn save_chunk(
    conn: &Connection,
    chunk_id: &str,
    document_id: &str,
    title: &str,
    text: &str,
    word_count: usize,
    preview: &str,
) -> SqliteResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO chunks
         (id, document_id, title, text, word_count, preview)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![chunk_id, document_id, title, text, word_count as i64, preview],
    )?;
    Ok(())
}

/// Add tokens to full-text index
pub fn add_fulltext_tokens(
    conn: &Connection,
    chunk_id: &str,
    tokens: &[(String, u32)],
) -> SqliteResult<()> {
    for (token, freq) in tokens {
        conn.execute(
            "INSERT OR REPLACE INTO fulltext_index (term, chunk_id, frequency)
             VALUES (?1, ?2, ?3)",
            params![token, chunk_id, *freq as i64],
        )?;
    }
    Ok(())
}

/// Add tokens to fuzzy index
pub fn add_fuzzy_tokens(
    conn: &Connection,
    chunk_id: &str,
    tokens: &[String],
) -> SqliteResult<()> {
    for token in tokens {
        conn.execute(
            "INSERT OR IGNORE INTO fuzzy_tokens (token, chunk_id)
             VALUES (?1, ?2)",
            params![token, chunk_id],
        )?;
    }
    Ok(())
}

/// Get all documents for index recovery
pub fn get_all_documents(conn: &Connection) -> SqliteResult<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, text FROM documents ORDER BY ingested_at"
    )?;

    let docs = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?
    .collect::<SqliteResult<Vec<_>>>()?;

    Ok(docs)
}

/// Get all chunks for a document
pub fn get_document_chunks(
    conn: &Connection,
    doc_id: &str,
) -> SqliteResult<Vec<(String, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, text FROM chunks WHERE document_id = ?1"
    )?;

    let chunks = stmt.query_map(params![doc_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?
    .collect::<SqliteResult<Vec<_>>>()?;

    Ok(chunks)
}

/// Get fulltext tokens for a chunk
pub fn get_fulltext_tokens(conn: &Connection, chunk_id: &str) -> SqliteResult<Vec<(String, u32)>> {
    let mut stmt = conn.prepare(
        "SELECT term, frequency FROM fulltext_index WHERE chunk_id = ?1"
    )?;

    let tokens = stmt.query_map(params![chunk_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u32))
    })?
    .collect::<SqliteResult<Vec<_>>>()?;

    Ok(tokens)
}

/// Get fuzzy tokens for a chunk
pub fn get_fuzzy_tokens(conn: &Connection, chunk_id: &str) -> SqliteResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT token FROM fuzzy_tokens WHERE chunk_id = ?1"
    )?;

    let tokens = stmt.query_map(params![chunk_id], |row| {
        row.get(0)
    })?
    .collect::<SqliteResult<Vec<_>>>()?;

    Ok(tokens)
}

/// Count total chunks in database
pub fn count_chunks(conn: &Connection) -> SqliteResult<usize> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM chunks")?;
    let count: i64 = stmt.query_row([], |row| row.get(0))?;
    Ok(count as usize)
}

/// Count total documents in database
pub fn count_documents(conn: &Connection) -> SqliteResult<usize> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM documents")?;
    let count: i64 = stmt.query_row([], |row| row.get(0))?;
    Ok(count as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_db() -> (Connection, String) {
        let path = "/tmp/test_const.db";
        let _ = fs::remove_file(path);
        let conn = open_db(path).unwrap();
        (conn, path.to_string())
    }

    #[test]
    fn test_init_schema() {
        let (conn, path) = setup_test_db();

        // Verify tables exist
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '%'"
        ).unwrap();

        let tables: Vec<String> = stmt.query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"documents".to_string()));
        assert!(tables.contains(&"chunks".to_string()));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_save_and_retrieve_document() {
        let (conn, path) = setup_test_db();

        save_document(
            &conn,
            "doc1",
            "Test Document",
            Some("Author"),
            Some("2026"),
            "test",
            None,
            "constitution",
            "Test content here",
        ).unwrap();

        let count = count_documents(&conn).unwrap();
        assert_eq!(count, 1);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_save_and_retrieve_chunk() {
        let (conn, path) = setup_test_db();

        save_document(&conn, "doc1", "Test", None, None, "test", None, "test", "content").unwrap();
        save_chunk(&conn, "chunk1", "doc1", "Chunk Title", "Chunk text", 50, "Chunk preview...").unwrap();

        let count = count_chunks(&conn).unwrap();
        assert_eq!(count, 1);

        let _ = fs::remove_file(&path);
    }
}
