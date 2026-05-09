//! WebAssembly bindings for `constitution-archive`.
//!
//! Native build: this is just a thin wrapper used by tests.
//! `wasm32` build: exposes a `WasmArchive` JS class via `wasm-bindgen`.
//!
//! Build:
//! ```text
//! wasm-pack build crates/constitution-wasm --release --target web
//! ```

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

use constitution_archive::{Archive, Filter, FilterValue, SearchHit, SearchOptions, Snippet};
#[cfg(target_arch = "wasm32")]
use constitution_archive::ProcessPhase;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// JS-side search request shape.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct JsSearchRequest {
    /// Query string.
    #[serde(default)]
    pub query: String,
    /// Maximum hits returned.
    #[serde(default)]
    pub limit: Option<usize>,
    /// Optional filter clauses.
    #[serde(default)]
    pub collections: Vec<String>,
    /// Optional author substrings.
    #[serde(default)]
    pub authors: Vec<String>,
    /// Optional document-type values.
    #[serde(default)]
    pub document_types: Vec<String>,
    /// Optional issue-tag values.
    #[serde(default)]
    pub issues: Vec<String>,
    /// Optional clause-tag values.
    #[serde(default)]
    pub clauses: Vec<String>,
    /// Optional date prefix (e.g. `"1787"`).
    #[serde(default)]
    pub date_prefix: Option<String>,
    /// Fuzzy expansion radius (Levenshtein). 0 disables.
    #[serde(default)]
    pub fuzzy_distance: Option<u32>,
    /// Snippet width in characters (0 suppresses snippets).
    #[serde(default)]
    pub snippet_window: Option<usize>,
}

impl JsSearchRequest {
    fn to_filter(&self) -> Filter {
        let mut f = Filter::default();
        if !self.collections.is_empty() {
            f = f.with(FilterValue::Collection(self.collections.clone()));
        }
        if !self.authors.is_empty() {
            f = f.with(FilterValue::Author(self.authors.clone()));
        }
        if !self.document_types.is_empty() {
            f = f.with(FilterValue::DocumentType(self.document_types.clone()));
        }
        if !self.issues.is_empty() {
            f = f.with(FilterValue::IssueTag(self.issues.clone()));
        }
        if !self.clauses.is_empty() {
            f = f.with(FilterValue::ClauseTag(self.clauses.clone()));
        }
        if let Some(p) = &self.date_prefix {
            f = f.with(FilterValue::DatePrefix(p.clone()));
        }
        f
    }

    fn to_options(&self) -> SearchOptions {
        SearchOptions {
            limit: self.limit.unwrap_or(25),
            min_score: 0.0,
            fuzzy_distance: self.fuzzy_distance.unwrap_or(0),
            snippet_window: self.snippet_window.unwrap_or(240),
        }
    }
}

/// Convenience search-result row sent back to JS.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsSearchHit {
    /// Stable chunk identifier.
    pub chunk_id: String,
    /// Document title.
    pub title: String,
    /// Author.
    pub author: String,
    /// Date.
    pub date: String,
    /// Source collection.
    pub collection: String,
    /// Source URL.
    pub source_url: String,
    /// First ~200 characters of the chunk (fallback when no snippet).
    pub preview: String,
    /// BM25 score.
    pub score: f32,
    /// Query terms that matched (post-fuzzy expansion).
    pub matched_terms: Vec<String>,
    /// KWIC snippet with highlight ranges (may be empty).
    pub snippet: Snippet,
}

fn enrich(archive: &Archive, hits: Vec<SearchHit>) -> Vec<JsSearchHit> {
    hits.into_iter()
        .filter_map(|h| {
            let c = archive.chunk(&h.chunk_id).ok()?;
            Some(JsSearchHit {
                chunk_id: h.chunk_id,
                title: c.title.clone(),
                author: c.author.clone(),
                date: c.date.clone(),
                collection: c.source_collection.clone(),
                source_url: c.source_url.clone(),
                preview: c.ensured_preview(),
                score: h.score,
                matched_terms: h.matched_terms,
                snippet: h.snippet,
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// wasm32 bindings
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
fn _wasm_init() {
    console_error_panic_hook::set_once();
}

/// JS-facing handle to a loaded archive.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmArchive {
    inner: Archive,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmArchive {
    /// Loads an archive from a `Uint8Array` of bytes.
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8]) -> Result<WasmArchive, JsError> {
        let inner = Archive::load(bytes).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner })
    }

    /// Number of chunks in the archive.
    #[wasm_bindgen(getter)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns archive stats as a JS object.
    pub fn stats(&self) -> Result<JsValue, JsError> {
        serde_wasm_bindgen::to_value(&self.inner.stats()).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Runs a BM25 search; `request_json` is the JSON form of [`JsSearchRequest`].
    pub fn search(&self, request_json: &str) -> Result<JsValue, JsError> {
        let req: JsSearchRequest =
            serde_json::from_str(request_json).map_err(|e| JsError::new(&e.to_string()))?;
        let raw = self
            .inner
            .search(&req.query, &req.to_filter(), &req.to_options());
        let enriched = enrich(&self.inner, raw);
        serde_wasm_bindgen::to_value(&enriched).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Fetches a single chunk by id and returns it as a JS object.
    pub fn chunk(&self, id: &str) -> Result<JsValue, JsError> {
        let c = self.inner.chunk(id).map_err(|e| JsError::new(&e.to_string()))?;
        serde_wasm_bindgen::to_value(c).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Returns the entire process timeline as a JS object.
    pub fn timeline(&self) -> Result<JsValue, JsError> {
        serde_wasm_bindgen::to_value(self.inner.timeline())
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Returns the timeline grouped by phase (label → events).
    pub fn timeline_by_phase(&self) -> Result<JsValue, JsError> {
        let map = self.inner.timeline_by_phase();
        serde_wasm_bindgen::to_value(&map).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Look up a single timeline event by id.
    pub fn process_event(&self, id: &str) -> Result<JsValue, JsError> {
        let e = self
            .inner
            .timeline()
            .get(id)
            .map_err(|e| JsError::new(&e.to_string()))?;
        serde_wasm_bindgen::to_value(e).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Returns events in a given phase ("convention", "ratification", …).
    pub fn process_phase(&self, phase: &str) -> Result<JsValue, JsError> {
        let p: ProcessPhase = serde_json::from_value(serde_json::Value::String(phase.to_string()))
            .map_err(|e| JsError::new(&e.to_string()))?;
        let events = self.inner.timeline().by_phase(p);
        serde_wasm_bindgen::to_value(&events).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Free-text search across the process timeline.
    pub fn process_search(&self, query: &str) -> Result<JsValue, JsError> {
        let events = self.inner.timeline().search(query);
        serde_wasm_bindgen::to_value(&events).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Returns the chunks associated with a given timeline event.
    pub fn chunks_for_event(&self, event_id: &str) -> Result<JsValue, JsError> {
        let chunks = self
            .inner
            .chunks_for_event(event_id)
            .map_err(|e| JsError::new(&e.to_string()))?;
        serde_wasm_bindgen::to_value(&chunks).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Type-ahead: indexed terms sharing a prefix.
    pub fn suggest(&self, prefix: &str, limit: usize) -> Result<JsValue, JsError> {
        let terms = self.inner.suggest(prefix, limit);
        serde_wasm_bindgen::to_value(&terms).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Fuzzy-match a single term, returning `[term, distance]` pairs.
    pub fn fuzzy_terms(
        &self,
        term: &str,
        max_distance: u32,
        limit: usize,
    ) -> Result<JsValue, JsError> {
        let hits = self.inner.fuzzy_terms(term, max_distance, limit);
        serde_wasm_bindgen::to_value(&hits).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Outgoing citations of a chunk: every clause / essay / person it
    /// references. Returns an array of `Citation` objects.
    pub fn citations_from(&self, chunk_id: &str) -> Result<JsValue, JsError> {
        let citations = self
            .inner
            .citations_from(chunk_id)
            .map_err(|e| JsError::new(&e.to_string()))?;
        serde_wasm_bindgen::to_value(&citations).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Every chunk that cites `target_key` (`"clause:I.8"`,
    /// `"essay:federalist:10"`, `"person:madison"`). Returns an array of
    /// `{ chunk: Chunk, citation: Citation }` objects.
    pub fn cited_by(&self, target_key: &str) -> Result<JsValue, JsError> {
        let pairs = self.inner.cited_by(target_key);
        let view: Vec<serde_json::Value> = pairs
            .into_iter()
            .map(|(chunk, citation)| {
                serde_json::json!({ "chunk": chunk, "citation": citation })
            })
            .collect();
        serde_wasm_bindgen::to_value(&view).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Top-N most-cited targets across the corpus.
    pub fn top_citation_targets(&self, limit: usize) -> Result<JsValue, JsError> {
        let top = self.inner.top_citation_targets(limit);
        serde_wasm_bindgen::to_value(&top).map_err(|e| JsError::new(&e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// Native helpers (used by tests + the CLI's integration flow)
// ---------------------------------------------------------------------------

/// Native helper: run a search and return enriched hits, no JS layer involved.
pub fn search_native(archive: &Archive, req: &JsSearchRequest) -> Vec<JsSearchHit> {
    let hits = archive.search(&req.query, &req.to_filter(), &req.to_options());
    enrich(archive, hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use constitution_archive::{Chunk, ProcessTimeline};

    fn fixture() -> Archive {
        let chunk = Chunk {
            chunk_id: "us_constitution_1787_article_1_0001".into(),
            document_id: "us_constitution_1787_article_1".into(),
            title: "Article I".into(),
            author: "Constitutional Convention".into(),
            date: "1787-09-17".into(),
            source_collection: "constitution".into(),
            source_url: "".into(),
            document_type: "foundational_document".into(),
            issue_tags: vec!["federalism".into()],
            constitutional_clause_tags: vec!["I.8".into()],
            text: "Congress shall have power to lay and collect Taxes".into(),
            word_count: 9,
            preview: "".into(),
        };
        Archive::build(vec![chunk], ProcessTimeline::default())
    }

    #[test]
    fn native_search_returns_results() {
        let archive = fixture();
        let req = JsSearchRequest {
            query: "Congress taxes".into(),
            collections: vec!["constitution".into()],
            ..Default::default()
        };
        let hits = search_native(&archive, &req);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].chunk_id, "us_constitution_1787_article_1_0001");
        assert!(!hits[0].preview.is_empty());
    }

    #[test]
    fn filter_excludes() {
        let archive = fixture();
        let req = JsSearchRequest {
            query: "Congress".into(),
            collections: vec!["federalist_papers".into()],
            ..Default::default()
        };
        assert!(search_native(&archive, &req).is_empty());
    }
}
