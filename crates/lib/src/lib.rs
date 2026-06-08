//! Constitutional Research System - Core Library
//!
//! This library provides high-performance indexing and search capabilities for
//! constitutional documents, with support for full-text search, fuzzy matching,
//! semantic search, and dynamic document ingestion.
//!
//! # Features
//!
//! - **Full-Text Indexing**: Inverted index with BM25 ranking
//! - **Fuzzy Matching**: BK-tree based sub-linear typo-tolerant search
//! - **Vector Search**: Semantic similarity using embeddings
//! - **Document Chunking**: Paragraph-aware with configurable overlap
//! - **Metadata Tagging**: Keyword-based taxonomy matching
//! - **Serialization**: Fast binary format (Bincode) and JSON export

pub mod chunker;
pub mod error;
pub mod fulltext_index;
pub mod fuzzy_match;
pub mod metadata_tagger;
pub mod tokenizer;
pub mod types;
pub mod vector_store;

// Re-export common types
pub use error::{Error, Result};
pub use types::{Chunk, ChunkId, Document, DocumentId, SearchResult};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
