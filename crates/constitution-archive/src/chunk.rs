//! Document chunk types matching the schema produced by the Python
//! ingest pipeline (`scripts/chunk_documents.py`).

use serde::{Deserialize, Serialize};

/// Stable string identifier for a chunk
/// (e.g. `us_constitution_1787_article_1_0000`).
pub type ChunkId = String;

/// A single passage of source text together with its bibliographic metadata.
///
/// Field names mirror `data/chunks/constitution_full_corpus.json`
/// so the JSON corpus can be ingested without translation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chunk {
    /// Stable identifier (`<document_id>_<offset>`).
    pub chunk_id: ChunkId,
    /// Identifier shared by every chunk that came from the same source document.
    pub document_id: String,
    /// Display title of the source document.
    pub title: String,
    /// Author or attributed body (may be `"Unknown"`).
    #[serde(default)]
    pub author: String,
    /// ISO-8601 calendar date if known, else free-text year/range.
    #[serde(default)]
    pub date: String,
    /// Top-level collection bucket (e.g. `constitution`, `federalist_papers`).
    #[serde(default)]
    pub source_collection: String,
    /// Public-domain source URL (Gutenberg, founders.archives.gov, etc.).
    #[serde(default)]
    pub source_url: String,
    /// Coarse document type tag (e.g. `foundational_document`, `letter`).
    #[serde(default)]
    pub document_type: String,
    /// Issue facets (federalism, separation_of_powers, …).
    #[serde(default)]
    pub issue_tags: Vec<String>,
    /// Constitutional-clause facets (e.g. `I.8`, `II.1`, `Amend.1`).
    #[serde(default)]
    pub constitutional_clause_tags: Vec<String>,
    /// Full chunk text.
    pub text: String,
    /// Cached word count (matches the Python pipeline's count).
    #[serde(default)]
    pub word_count: u32,
    /// First ~200 characters of the chunk; built lazily if missing.
    #[serde(default)]
    pub preview: String,
}

impl Chunk {
    /// Returns a non-empty preview, computing one from `text` if necessary.
    pub fn ensured_preview(&self) -> String {
        if !self.preview.is_empty() {
            return self.preview.clone();
        }
        let mut out: String = self.text.chars().take(200).collect();
        if self.text.chars().count() > 200 {
            out.push('…');
        }
        out
    }
}
