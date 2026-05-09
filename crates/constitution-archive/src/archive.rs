//! Top-level archive: chunks + inverted index + process timeline.
//!
//! ## Binary format
//!
//! Bincode-serialized [`ArchivePayload`] with a 4-byte magic header
//! `b"CARC"` and a 4-byte little-endian version (currently `1`).
//! Loaders verify both before deserializing.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::chunk::{Chunk, ChunkId};
use crate::error::ArchiveError;
use crate::filter::Filter;
use crate::index::{InvertedIndex, SearchHit, SearchOptions};
use crate::process::{ProcessEvent, ProcessPhase, ProcessTimeline};

/// Magic header for archive blobs.
pub const ARCHIVE_MAGIC: &[u8; 4] = b"CARC";
/// Current archive format version.
pub const ARCHIVE_VERSION: u32 = 1;

/// Aggregate statistics for `Archive::stats()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveStats {
    /// Total chunks in the corpus.
    pub chunks: usize,
    /// Distinct documents.
    pub documents: usize,
    /// Distinct terms in the inverted index.
    pub terms: usize,
    /// Total events in the process timeline.
    pub events: usize,
    /// Distinct collections (e.g. `constitution`, `federalist_papers`).
    pub collections: usize,
    /// Distinct authors.
    pub authors: usize,
}

/// What is actually serialized to disk / sent to the browser.
#[derive(Debug, Serialize, Deserialize)]
struct ArchivePayload {
    chunks: Vec<Chunk>,
    index: InvertedIndex,
    timeline: ProcessTimeline,
}

/// Loaded, queryable archive.
#[derive(Debug)]
pub struct Archive {
    chunks: Vec<Chunk>,
    chunk_index_by_id: HashMap<ChunkId, u32>,
    index: InvertedIndex,
    timeline: ProcessTimeline,
}

impl Archive {
    /// Builds an archive from a chunk vector and an optional process timeline.
    ///
    /// The inverted index is constructed in this call (O(total tokens)).
    pub fn build(chunks: Vec<Chunk>, timeline: ProcessTimeline) -> Self {
        let index = InvertedIndex::build(
            chunks
                .iter()
                .enumerate()
                .map(|(i, c)| (i as u32, c.text.as_str())),
        );
        let chunk_index_by_id = chunks
            .iter()
            .enumerate()
            .map(|(i, c)| (c.chunk_id.clone(), i as u32))
            .collect();
        Self {
            chunks,
            chunk_index_by_id,
            index,
            timeline,
        }
    }

    /// Loads an archive from its binary form (magic + version + bincode).
    pub fn load(bytes: &[u8]) -> Result<Self, ArchiveError> {
        if bytes.len() < 8 || &bytes[..4] != ARCHIVE_MAGIC {
            return Err(ArchiveError::Malformed("bad magic".into()));
        }
        let version = u32::from_le_bytes(bytes[4..8].try_into().map_err(|_| {
            ArchiveError::Malformed("bad version header".into())
        })?);
        if version != ARCHIVE_VERSION {
            return Err(ArchiveError::UnsupportedVersion(version));
        }
        let payload: ArchivePayload = bincode::deserialize(&bytes[8..])?;
        let chunk_index_by_id = payload
            .chunks
            .iter()
            .enumerate()
            .map(|(i, c)| (c.chunk_id.clone(), i as u32))
            .collect();
        Ok(Self {
            chunks: payload.chunks,
            chunk_index_by_id,
            index: payload.index,
            timeline: payload.timeline,
        })
    }

    /// Serializes the archive to its binary form.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ArchiveError> {
        let payload = ArchivePayload {
            chunks: self.chunks.clone(),
            index: self.index.clone(),
            timeline: self.timeline.clone(),
        };
        let body = bincode::serialize(&payload)?;
        let mut out = Vec::with_capacity(8 + body.len());
        out.extend_from_slice(ARCHIVE_MAGIC);
        out.extend_from_slice(&ARCHIVE_VERSION.to_le_bytes());
        out.extend_from_slice(&body);
        Ok(out)
    }

    /// Number of chunks in the corpus.
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// Returns `true` if the corpus contains no chunks.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// Look up a chunk by its stable id.
    pub fn chunk(&self, id: &str) -> Result<&Chunk, ArchiveError> {
        let pos = self
            .chunk_index_by_id
            .get(id)
            .ok_or_else(|| ArchiveError::ChunkNotFound(id.to_string()))?;
        self.chunks
            .get(*pos as usize)
            .ok_or_else(|| ArchiveError::ChunkNotFound(id.to_string()))
    }

    /// Borrow the full chunk slice (for export / iteration).
    pub fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }

    /// Borrow the timeline.
    pub fn timeline(&self) -> &ProcessTimeline {
        &self.timeline
    }

    /// BM25 search with optional metadata filter.
    pub fn search(&self, query: &str, filter: &Filter, opts: &SearchOptions) -> Vec<SearchHit> {
        let raw = self.index.search(query, opts, |idx| {
            let Some(c) = self.chunks.get(idx as usize) else {
                return false;
            };
            filter.matches(c)
        });
        raw.into_iter()
            .map(|(idx, score, terms)| SearchHit {
                chunk_id: self
                    .chunks
                    .get(idx as usize)
                    .map(|c| c.chunk_id.clone())
                    .unwrap_or_default(),
                score,
                matched_terms: terms,
            })
            .collect()
    }

    /// Returns events about a particular chunk.
    pub fn events_for_chunk(&self, chunk_id: &str) -> Vec<&ProcessEvent> {
        self.timeline
            .events
            .iter()
            .filter(|e| e.source_chunks.iter().any(|c| c == chunk_id))
            .collect()
    }

    /// Returns chunks cited by a particular process event (in archive order).
    pub fn chunks_for_event(&self, event_id: &str) -> Result<Vec<&Chunk>, ArchiveError> {
        let event = self.timeline.get(event_id)?;
        let mut out = Vec::with_capacity(event.source_chunks.len());
        for cid in &event.source_chunks {
            if let Ok(c) = self.chunk(cid) {
                out.push(c);
            }
        }
        Ok(out)
    }

    /// Phase-by-phase grouping of timeline events for UI rendering.
    pub fn timeline_by_phase(&self) -> BTreeMap<&'static str, Vec<&ProcessEvent>> {
        let phases = [
            ProcessPhase::Antecedents,
            ProcessPhase::Convention,
            ProcessPhase::Ratification,
            ProcessPhase::BillOfRightsDrafting,
            ProcessPhase::BillOfRightsRatification,
        ];
        phases
            .iter()
            .map(|p| (p.label(), self.timeline.by_phase(*p)))
            .collect()
    }

    /// Cheap summary statistics over the loaded archive.
    pub fn stats(&self) -> ArchiveStats {
        let mut docs = std::collections::HashSet::new();
        let mut cols = std::collections::HashSet::new();
        let mut auths = std::collections::HashSet::new();
        for c in &self.chunks {
            docs.insert(c.document_id.as_str());
            cols.insert(c.source_collection.as_str());
            auths.insert(c.author.as_str());
        }
        ArchiveStats {
            chunks: self.chunks.len(),
            documents: docs.len(),
            terms: self.index.terms.len(),
            events: self.timeline.len(),
            collections: cols.len(),
            authors: auths.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::Chunk;

    fn chunk(id: &str, text: &str) -> Chunk {
        Chunk {
            chunk_id: id.into(),
            document_id: id.into(),
            title: "T".into(),
            author: "Madison".into(),
            date: "1787-09-17".into(),
            source_collection: "constitution".into(),
            source_url: "".into(),
            document_type: "foundational_document".into(),
            issue_tags: vec!["federalism".into()],
            constitutional_clause_tags: vec!["I.1".into()],
            text: text.into(),
            word_count: text.split_whitespace().count() as u32,
            preview: "".into(),
        }
    }

    #[test]
    fn build_search_roundtrip() {
        let chunks = vec![
            chunk("a", "We the People of the United States"),
            chunk("b", "Congress shall make no law respecting religion"),
        ];
        let archive = Archive::build(chunks, ProcessTimeline::default());
        let hits = archive.search("religion", &Filter::default(), &SearchOptions::default());
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].chunk_id, "b");

        let bytes = archive.to_bytes().unwrap();
        let reloaded = Archive::load(&bytes).unwrap();
        assert_eq!(reloaded.len(), 2);
        let hits2 = reloaded.search("religion", &Filter::default(), &SearchOptions::default());
        assert_eq!(hits2.len(), 1);
    }

    #[test]
    fn rejects_bad_magic() {
        let err = Archive::load(b"NOPE0000extra").unwrap_err();
        assert!(matches!(err, ArchiveError::Malformed(_)));
    }

    #[test]
    fn rejects_bad_version() {
        let mut bytes = b"CARC".to_vec();
        bytes.extend_from_slice(&999u32.to_le_bytes());
        let err = Archive::load(&bytes).unwrap_err();
        assert!(matches!(err, ArchiveError::UnsupportedVersion(999)));
    }
}
