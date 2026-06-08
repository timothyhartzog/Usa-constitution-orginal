//! Error types for the constitutional research library

use thiserror::Error;

/// Result type alias using the Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Library error types
#[derive(Error, Debug)]
pub enum Error {
    #[error("Tokenization error: {0}")]
    TokenizationError(String),

    #[error("Indexing error: {0}")]
    IndexingError(String),

    #[error("Search error: {0}")]
    SearchError(String),

    #[error("Chunk not found: {0}")]
    ChunkNotFound(String),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Invalid document: {0}")]
    InvalidDocument(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Invalid chunk strategy: {0}")]
    InvalidChunkStrategy(String),

    #[error("Invalid embeddings: {0}")]
    InvalidEmbeddings(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Bincode error: {0}")]
    BincodeError(#[from] bincode::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Other(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Other(s.to_string())
    }
}
