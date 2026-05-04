//! Full-text search indexing with BM25 ranking
//!
//! Implements an inverted index for O(1) term lookup and BM25 scoring
//! for relevance-based ranking of search results.

use crate::error::Result;
use crate::types::{Chunk, ChunkId};
use std::collections::HashMap;

/// BM25 parameters (default tuning values)
const BM25_K1: f32 = 1.5;
const BM25_B: f32 = 0.75;

/// Full-text search index using inverted index and BM25 ranking
#[derive(Debug, Clone)]
pub struct FullTextIndex {
    /// Inverted index: term -> list of chunk IDs containing term
    inverted_index: HashMap<String, Vec<ChunkId>>,
    /// Term frequencies: (term, chunk_id) -> frequency
    term_frequencies: HashMap<(String, ChunkId), u32>,
    /// Document lengths (word count per chunk)
    doc_lengths: HashMap<ChunkId, u32>,
    /// Total number of documents
    total_docs: u32,
    /// Average document length
    avg_doc_length: f32,
}

impl Default for FullTextIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl FullTextIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self {
            inverted_index: HashMap::new(),
            term_frequencies: HashMap::new(),
            doc_lengths: HashMap::new(),
            total_docs: 0,
            avg_doc_length: 0.0,
        }
    }

    /// Add a chunk to the index
    pub fn add_chunk(&mut self, chunk_id: ChunkId, tokens: Vec<String>) -> Result<()> {
        let doc_length = tokens.len() as u32;

        // Update document length
        self.doc_lengths.insert(chunk_id.clone(), doc_length);

        // Update average document length
        let prev_total: u32 = self.doc_lengths.values().sum();
        self.total_docs = self.doc_lengths.len() as u32;
        self.avg_doc_length = prev_total as f32 / self.total_docs.max(1) as f32;

        // Index tokens
        let mut term_freq: HashMap<String, u32> = HashMap::new();
        for token in tokens {
            *term_freq.entry(token.clone()).or_insert(0) += 1;
            self.inverted_index
                .entry(token)
                .or_insert_with(Vec::new)
                .push(chunk_id.clone());
        }

        // Store term frequencies
        for (term, freq) in term_freq {
            self.term_frequencies
                .insert((term, chunk_id.clone()), freq);
        }

        Ok(())
    }

    /// Search for chunks matching all query terms using BM25 ranking
    pub fn search(&self, query_tokens: &[String]) -> Result<Vec<(ChunkId, f32)>> {
        if query_tokens.is_empty() {
            return Ok(vec![]);
        }

        // Get chunks matching all terms (intersection)
        let mut candidate_chunks: Option<Vec<ChunkId>> = None;

        for token in query_tokens {
            let matching_chunks = self
                .inverted_index
                .get(token)
                .cloned()
                .unwrap_or_default();

            candidate_chunks = match candidate_chunks {
                None => Some(matching_chunks),
                Some(existing) => {
                    let existing_set: std::collections::HashSet<_> =
                        existing.into_iter().collect();
                    let filtered: Vec<_> = matching_chunks
                        .into_iter()
                        .filter(|c| existing_set.contains(c))
                        .collect();
                    Some(filtered)
                }
            };
        }

        let mut results = Vec::new();

        // Score each candidate chunk using BM25
        for chunk_id in candidate_chunks.unwrap_or_default() {
            let score = self.bm25_score(&chunk_id, query_tokens)?;
            results.push((chunk_id, score));
        }

        // Sort by score descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(results)
    }

    /// Calculate BM25 score for a chunk
    fn bm25_score(&self, chunk_id: &ChunkId, query_tokens: &[String]) -> Result<f32> {
        let doc_length = *self.doc_lengths.get(chunk_id).unwrap_or(&0) as f32;
        let mut score = 0.0;

        for token in query_tokens {
            let idf = self.calculate_idf(token)?;
            let tf = *self
                .term_frequencies
                .get(&(token.clone(), chunk_id.clone()))
                .unwrap_or(&0) as f32;

            let numerator = tf * (BM25_K1 + 1.0);
            let denominator = tf
                + BM25_K1
                    * (1.0 - BM25_B + BM25_B * (doc_length / self.avg_doc_length.max(1.0)));

            score += idf * (numerator / denominator);
        }

        Ok(score)
    }

    /// Calculate inverse document frequency for a term
    fn calculate_idf(&self, term: &str) -> Result<f32> {
        let docs_with_term = self
            .inverted_index
            .get(term)
            .map(|v| v.len() as f32)
            .unwrap_or(0.0);

        let idf = ((self.total_docs as f32 - docs_with_term + 0.5) / (docs_with_term + 0.5) + 1.0)
            .ln();
        Ok(idf)
    }

    /// Get statistics about the index
    pub fn stats(&self) -> (usize, usize, u32, f32) {
        (
            self.inverted_index.len(),
            self.doc_lengths.len(),
            self.total_docs,
            self.avg_doc_length,
        )
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.inverted_index.clear();
        self.term_frequencies.clear();
        self.doc_lengths.clear();
        self.total_docs = 0;
        self.avg_doc_length = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_chunk() {
        let mut index = FullTextIndex::new();
        let chunk_id = ChunkId::new("doc1_0000");
        let tokens = vec!["constitution".to_string(), "ratified".to_string()];

        assert!(index.add_chunk(chunk_id.clone(), tokens).is_ok());
        let (terms, docs, _, _) = index.stats();
        assert_eq!(terms, 2);
        assert_eq!(docs, 1);
    }

    #[test]
    fn test_search_single_term() {
        let mut index = FullTextIndex::new();
        let chunk_id = ChunkId::new("doc1_0000");
        let tokens = vec!["constitution".to_string(), "ratified".to_string()];
        index.add_chunk(chunk_id.clone(), tokens).unwrap();

        let results = index
            .search(&["constitution".to_string()])
            .unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].0, chunk_id);
    }

    #[test]
    fn test_search_intersection() {
        let mut index = FullTextIndex::new();

        let chunk1 = ChunkId::new("doc1_0000");
        let chunk2 = ChunkId::new("doc2_0000");

        index
            .add_chunk(
                chunk1.clone(),
                vec!["constitution".to_string(), "ratified".to_string()],
            )
            .unwrap();
        index
            .add_chunk(
                chunk2,
                vec!["constitution".to_string(), "amended".to_string()],
            )
            .unwrap();

        // Search for both terms - should only match chunk1
        let results = index
            .search(&["constitution".to_string(), "ratified".to_string()])
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, chunk1);
    }
}
