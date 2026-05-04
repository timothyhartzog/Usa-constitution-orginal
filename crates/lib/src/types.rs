//! Core type definitions used throughout the library

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a document
#[derive(
    Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct DocumentId(pub String);

impl DocumentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl AsRef<str> for DocumentId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a chunk within a document
#[derive(
    Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct ChunkId(pub String);

impl ChunkId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Create a chunk ID from document ID and sequence number
    pub fn from_doc_and_seq(doc_id: &DocumentId, seq: usize) -> Self {
        Self(format!("{}_{:04}", doc_id.0, seq))
    }
}

impl AsRef<str> for ChunkId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadata about a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub title: String,
    pub author: Option<String>,
    pub date: Option<String>,
    pub source_collection: String,
    pub source_url: Option<String>,
    pub document_type: String,
    pub text: String,
    pub metadata: HashMap<String, String>,
}

/// Chunking strategy for splitting documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkStrategy {
    /// Split on constitutional article markers
    ConstitutionSections,
    /// Split on "FEDERALIST No." headers
    FederalistEssays,
    /// Split on "TO {NAME}" patterns
    JeffersonLetters,
    /// Generic sliding window with configurable parameters
    SlidingWindow {
        target_words: usize,
        min_words: usize,
        max_words: usize,
        overlap_words: usize,
    },
}

impl Default for ChunkStrategy {
    fn default() -> Self {
        Self::SlidingWindow {
            target_words: 320,
            min_words: 80,
            max_words: 650,
            overlap_words: 45,
        }
    }
}

/// A chunk of text from a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: ChunkId,
    pub document_id: DocumentId,
    pub title: String,
    pub text: String,
    pub word_count: usize,
    pub preview: String,
    pub issue_tags: Vec<String>,
    pub clause_tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Chunk {
    /// Create preview text (first N words with ellipsis)
    pub fn make_preview(text: &str, max_words: usize) -> String {
        let words: Vec<&str> = text.split_whitespace().take(max_words).collect();
        let preview = words.join(" ");
        if text.split_whitespace().count() > max_words {
            format!("{}…", preview)
        } else {
            preview
        }
    }
}

/// Result of a search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk: Chunk,
    pub score: f32,
    pub search_type: SearchType,
}

/// Type of search performed
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SearchType {
    FullText,
    Fuzzy,
    Semantic,
    Combined,
}

impl std::fmt::Display for SearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchType::FullText => write!(f, "full-text"),
            SearchType::Fuzzy => write!(f, "fuzzy"),
            SearchType::Semantic => write!(f, "semantic"),
            SearchType::Combined => write!(f, "combined"),
        }
    }
}

/// Search filters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    pub collections: Option<Vec<String>>,
    pub document_ids: Option<Vec<DocumentId>>,
    pub authors: Option<Vec<String>>,
    pub issue_tags: Option<Vec<String>>,
    pub clause_tags: Option<Vec<String>>,
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_documents: usize,
    pub total_chunks: usize,
    pub total_terms: usize,
    pub avg_chunk_size: f32,
    pub index_size_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_id_creation() {
        let doc_id = DocumentId::new("test_doc");
        let chunk_id = ChunkId::from_doc_and_seq(&doc_id, 5);
        assert_eq!(chunk_id.0, "test_doc_0005");
    }

    #[test]
    fn test_chunk_preview() {
        let text = "This is a long text with many words that should be truncated";
        let preview = Chunk::make_preview(text, 5);
        assert!(preview.contains("…"));
        assert_eq!(preview, "This is a long text…");
    }

    #[test]
    fn test_default_chunk_strategy() {
        let strategy = ChunkStrategy::default();
        match strategy {
            ChunkStrategy::SlidingWindow {
                target_words,
                min_words,
                max_words,
                overlap_words,
            } => {
                assert_eq!(target_words, 320);
                assert_eq!(min_words, 80);
                assert_eq!(max_words, 650);
                assert_eq!(overlap_words, 45);
            }
            _ => panic!("Expected SlidingWindow strategy"),
        }
    }
}
