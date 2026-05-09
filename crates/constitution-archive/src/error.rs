//! Error types for the constitution archive.

use thiserror::Error;

/// Errors raised by the archive crate.
#[derive(Debug, Error)]
pub enum ArchiveError {
    /// Wraps a `serde_json` parse error during corpus ingestion.
    #[error("invalid corpus JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// Wraps a `bincode` (de)serialization error for the binary archive bytes.
    #[error("archive bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    /// The archive bytes carry an unsupported format version.
    #[error("unsupported archive format version: {0}")]
    UnsupportedVersion(u32),

    /// A chunk identifier was referenced that is not present in the archive.
    #[error("chunk not found: {0}")]
    ChunkNotFound(String),

    /// A process event id was referenced that is not present in the timeline.
    #[error("process event not found: {0}")]
    ProcessEventNotFound(String),

    /// The supplied corpus document was missing a required field.
    #[error("malformed corpus: {0}")]
    Malformed(String),
}
