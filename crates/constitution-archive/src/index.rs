//! Inverted index with BM25 ranking.
//!
//! Designed for an offline corpus that fits in memory:
//!  - `terms`: term → posting list (chunk-index, term-frequency)
//!  - `doc_lens`: per-chunk total token count (for BM25 normalization)
//!  - `avg_dlen`: average document length over the corpus
//!
//! Native and `wasm32` builds share this implementation byte-for-byte.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::tokenizer::tokenize;

/// BM25 free parameter `k1` — controls term-frequency saturation.
pub const BM25_K1: f32 = 1.5;
/// BM25 free parameter `b` — controls length normalization.
pub const BM25_B: f32 = 0.75;

/// One posting in the inverted index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Posting {
    /// Position of the chunk in [`crate::Archive::chunks`].
    pub chunk_idx: u32,
    /// Number of times the term occurs in that chunk.
    pub tf: u32,
}

/// Search-time tunables.
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Maximum number of hits returned (after filtering).
    pub limit: usize,
    /// Minimum BM25 score; hits below this are dropped.
    pub min_score: f32,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 25,
            min_score: 0.0,
        }
    }
}

/// A scored search hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    /// Stable chunk identifier.
    pub chunk_id: String,
    /// BM25 score for the matching query.
    pub score: f32,
    /// Comma-separated list of query terms that matched the chunk.
    pub matched_terms: Vec<String>,
}

/// In-memory inverted index over the chunk corpus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvertedIndex {
    /// term → postings
    pub terms: HashMap<String, Vec<Posting>>,
    /// per-chunk token count
    pub doc_lens: Vec<u32>,
    /// average doc length over the corpus
    pub avg_dlen: f32,
}

impl InvertedIndex {
    /// Builds the inverted index over an iterator of `(chunk_idx, text)` pairs.
    pub fn build<'a, I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (u32, &'a str)>,
    {
        let mut terms: HashMap<String, HashMap<u32, u32>> = HashMap::new();
        let mut doc_lens: Vec<u32> = Vec::new();

        for (idx, text) in iter {
            let toks = tokenize(text);
            let dlen = toks.len() as u32;
            // grow doc_lens to fit
            if (idx as usize) >= doc_lens.len() {
                doc_lens.resize((idx as usize) + 1, 0);
            }
            doc_lens[idx as usize] = dlen;
            for tok in toks {
                *terms.entry(tok).or_default().entry(idx).or_insert(0) += 1;
            }
        }

        let total: u64 = doc_lens.iter().map(|&l| l as u64).sum();
        let avg_dlen = if doc_lens.is_empty() {
            0.0
        } else {
            (total as f32) / (doc_lens.len() as f32)
        };

        let terms = terms
            .into_iter()
            .map(|(t, m)| {
                let mut postings: Vec<Posting> = m
                    .into_iter()
                    .map(|(chunk_idx, tf)| Posting { chunk_idx, tf })
                    .collect();
                postings.sort_by_key(|p| p.chunk_idx);
                (t, postings)
            })
            .collect();

        Self {
            terms,
            doc_lens,
            avg_dlen,
        }
    }

    /// Number of indexed chunks.
    pub fn doc_count(&self) -> usize {
        self.doc_lens.len()
    }

    /// Returns `true` if the index has no chunks indexed.
    pub fn is_empty(&self) -> bool {
        self.doc_lens.is_empty()
    }

    /// Runs a BM25 query and returns chunk indices with scores and matched terms.
    ///
    /// Filtering by metadata is done by the caller via the `accept` predicate.
    /// `accept` is invoked **before** scoring is finalized so we never compute
    /// BM25 contributions for a chunk that will be discarded.
    pub fn search<F>(&self, query: &str, opts: &SearchOptions, accept: F) -> Vec<(u32, f32, Vec<String>)>
    where
        F: Fn(u32) -> bool,
    {
        let qtoks = tokenize(query);
        if qtoks.is_empty() || self.is_empty() {
            return Vec::new();
        }

        let n = self.doc_lens.len() as f32;
        let mut acc: HashMap<u32, (f32, Vec<String>)> = HashMap::new();

        for term in qtoks {
            let Some(postings) = self.terms.get(&term) else {
                continue;
            };
            let df = postings.len() as f32;
            // BM25 IDF with the standard "+1" smoothing: avoids negative values
            // and the singular point at df = n/2.
            let idf = (((n - df + 0.5) / (df + 0.5)) + 1.0).ln();
            for p in postings {
                if !accept(p.chunk_idx) {
                    continue;
                }
                let dl = self.doc_lens[p.chunk_idx as usize] as f32;
                let denom =
                    (p.tf as f32) + BM25_K1 * (1.0 - BM25_B + BM25_B * dl / self.avg_dlen.max(1.0));
                let contrib = idf * ((p.tf as f32) * (BM25_K1 + 1.0)) / denom.max(1e-6);
                let entry = acc.entry(p.chunk_idx).or_insert_with(|| (0.0, Vec::new()));
                entry.0 += contrib;
                if !entry.1.contains(&term) {
                    entry.1.push(term.clone());
                }
            }
        }

        let mut hits: Vec<(u32, f32, Vec<String>)> = acc
            .into_iter()
            .filter(|(_, (s, _))| *s >= opts.min_score)
            .map(|(idx, (score, terms))| (idx, score, terms))
            .collect();
        hits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        hits.truncate(opts.limit);
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_search_smoke() {
        let docs = [
            (0u32, "We the People of the United States, in Order to form a more perfect Union"),
            (1, "Congress shall make no law respecting an establishment of religion"),
            (2, "The right of the people to keep and bear arms"),
        ];
        let idx = InvertedIndex::build(docs.iter().map(|(i, t)| (*i, *t)));

        let opts = SearchOptions::default();
        let hits = idx.search("religion establishment", &opts, |_| true);
        assert!(!hits.is_empty());
        assert_eq!(hits[0].0, 1);
        assert!(hits[0].1 > 0.0);

        // accept-predicate filters out a chunk
        let hits = idx.search("people", &opts, |i| i != 0);
        assert!(hits.iter().all(|(i, _, _)| *i != 0));
    }

    #[test]
    fn empty_query_returns_nothing() {
        let idx = InvertedIndex::build([(0u32, "foo bar baz")]);
        let hits = idx.search("", &SearchOptions::default(), |_| true);
        assert!(hits.is_empty());
    }
}
