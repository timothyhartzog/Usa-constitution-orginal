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
use crate::citation::{Citation, CitationGraph, CitationGraphView};
use crate::error::ArchiveError;
use crate::filter::Filter;
use crate::fuzzy::BkTree;
use crate::index::{InvertedIndex, SearchHit, SearchOptions};
use crate::process::{ProcessEvent, ProcessPhase, ProcessTimeline};
use crate::snippet::{make_snippet, Snippet};

/// Cap on how many fuzzy alternatives are tried per query term — prevents a
/// single typo from exploding into hundreds of expanded postings.
const MAX_FUZZY_ALTERNATIVES: usize = 3;

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
    /// Total citations extracted from chunk text.
    pub citations: usize,
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
    /// BK-tree over the inverted-index vocabulary. Built lazily during
    /// `build()`/`load()` — never serialized, so the binary archive format
    /// stays unchanged.
    bk_tree: BkTree,
    /// Citation graph extracted from chunk text (clauses + essays + persons).
    /// Like `bk_tree`, derived on construction so the archive format is unchanged.
    citation_graph: CitationGraph,
}

fn bk_tree_from(index: &InvertedIndex) -> BkTree {
    BkTree::from_terms(index.vocabulary().map(str::to_string))
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
        let bk_tree = bk_tree_from(&index);
        let citation_graph = CitationGraph::build(&chunks);
        Self {
            chunks,
            chunk_index_by_id,
            index,
            timeline,
            bk_tree,
            citation_graph,
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
        let bk_tree = bk_tree_from(&payload.index);
        let citation_graph = CitationGraph::build(&payload.chunks);
        Ok(Self {
            chunks: payload.chunks,
            chunk_index_by_id,
            index: payload.index,
            timeline: payload.timeline,
            bk_tree,
            citation_graph,
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

    /// BM25 search with optional fuzzy expansion, metadata filter, and snippets.
    pub fn search(&self, query: &str, filter: &Filter, opts: &SearchOptions) -> Vec<SearchHit> {
        let bk = &self.bk_tree;
        let fuzzy_distance = opts.fuzzy_distance;
        let raw = self.index.search(
            query,
            opts,
            |idx| {
                let Some(c) = self.chunks.get(idx as usize) else {
                    return false;
                };
                filter.matches(c)
            },
            |term| {
                if fuzzy_distance == 0 {
                    return Vec::new();
                }
                bk.search(term, fuzzy_distance, MAX_FUZZY_ALTERNATIVES)
                    .into_iter()
                    .map(|(t, _)| t)
                    .collect()
            },
        );
        raw.into_iter()
            .map(|(idx, score, terms)| {
                let chunk_id = self
                    .chunks
                    .get(idx as usize)
                    .map(|c| c.chunk_id.clone())
                    .unwrap_or_default();
                let snippet = if opts.snippet_window == 0 {
                    Snippet::empty()
                } else {
                    self.chunks
                        .get(idx as usize)
                        .map(|c| make_snippet(&c.text, &terms, opts.snippet_window))
                        .unwrap_or_default()
                };
                SearchHit {
                    chunk_id,
                    score,
                    matched_terms: terms,
                    snippet,
                }
            })
            .collect()
    }

    /// Returns up to `limit` indexed terms that share `prefix`
    /// (case-insensitive). Useful for type-ahead suggestions.
    pub fn suggest(&self, prefix: &str, limit: usize) -> Vec<String> {
        if prefix.is_empty() || limit == 0 {
            return Vec::new();
        }
        let needle = prefix.to_lowercase();
        let mut hits: Vec<&str> = self
            .index
            .vocabulary()
            .filter(|t| t.starts_with(&needle))
            .collect();
        hits.sort_unstable();
        hits.into_iter().take(limit).map(String::from).collect()
    }

    /// Returns up to `limit` indexed terms within `max_distance`
    /// of `term` according to the BK-tree (for typo correction UIs).
    pub fn fuzzy_terms(&self, term: &str, max_distance: u32, limit: usize) -> Vec<(String, u32)> {
        self.bk_tree.search(term, max_distance, limit)
    }

    /// Borrow the citation graph.
    pub fn citation_graph(&self) -> &CitationGraph {
        &self.citation_graph
    }

    /// Returns the outgoing citations of a chunk
    /// (everything the chunk text references).
    pub fn citations_from(&self, chunk_id: &str) -> Result<Vec<&Citation>, ArchiveError> {
        let idx = self
            .chunk_index_by_id
            .get(chunk_id)
            .ok_or_else(|| ArchiveError::ChunkNotFound(chunk_id.to_string()))?;
        Ok(self.citation_graph.out_citations(*idx))
    }

    /// Returns every chunk that cites a given target.
    ///
    /// `target_key` uses the canonical form `"clause:I.8"`,
    /// `"essay:federalist:10"`, or `"person:madison"`.
    pub fn cited_by(&self, target_key: &str) -> Vec<(&Chunk, &Citation)> {
        self.citation_graph
            .cited_by(target_key)
            .into_iter()
            .filter_map(|c| self.chunks.get(c.from_chunk as usize).map(|chunk| (chunk, c)))
            .collect()
    }

    /// Returns the most-cited targets across the corpus.
    pub fn top_citation_targets(&self, limit: usize) -> Vec<(String, usize)> {
        self.citation_graph.top_targets(limit)
    }

    /// Returns a renderable co-occurrence view of the top-`top_n`
    /// citation targets.
    pub fn citation_graph_view(&self, top_n: usize) -> CitationGraphView {
        self.citation_graph.graph_view(top_n)
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
            citations: self.citation_graph.len(),
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

    #[test]
    fn suggest_returns_prefix_matches_only() {
        let chunks = vec![
            chunk("a", "ratify ratification ratifying ratifies federalism"),
            chunk("b", "constitution convention"),
        ];
        let archive = Archive::build(chunks, ProcessTimeline::default());
        let mut hits = archive.suggest("ratif", 10);
        hits.sort();
        assert_eq!(
            hits,
            vec!["ratification", "ratifies", "ratify", "ratifying"]
        );
    }

    #[test]
    fn fuzzy_search_recovers_typo() {
        let chunks = vec![chunk("a", "the great compromise of 1787 settled representation")];
        let archive = Archive::build(chunks, ProcessTimeline::default());
        // Plain search should miss.
        let none = archive.search(
            "comprmise",
            &Filter::default(),
            &SearchOptions {
                fuzzy_distance: 0,
                ..SearchOptions::default()
            },
        );
        assert!(none.is_empty());
        // With fuzzy expansion enabled it should land.
        let hits = archive.search(
            "comprmise",
            &Filter::default(),
            &SearchOptions {
                fuzzy_distance: 2,
                ..SearchOptions::default()
            },
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].chunk_id, "a");
        assert!(hits[0].matched_terms.iter().any(|t| t == "compromise"));
    }

    #[test]
    fn search_populates_snippet_with_highlights() {
        let chunks = vec![chunk(
            "x",
            "Congress shall make no law respecting an establishment of religion, or prohibiting the free exercise thereof.",
        )];
        let archive = Archive::build(chunks, ProcessTimeline::default());
        let hits = archive.search("religion exercise", &Filter::default(), &SearchOptions::default());
        assert_eq!(hits.len(), 1);
        assert!(!hits[0].snippet.text.is_empty());
        assert_eq!(hits[0].snippet.highlights.len(), 2);
    }
}
