use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Ordering;
use serde_json::json;

use crate::{Chunk, SearchResult, CorpusStats};
use crate::utils::{tokenize, normalize};

#[derive(Debug)]
pub struct SearchEngine {
    chunks: HashMap<String, Chunk>,
    inverted_index: HashMap<String, Vec<String>>, // word -> chunk_ids
    clause_index: HashMap<String, Vec<String>>,   // clause -> chunk_ids
    issue_index: HashMap<String, Vec<String>>,    // issue -> chunk_ids
    collection_index: HashMap<String, Vec<String>>, // collection -> chunk_ids
    author_index: HashMap<String, Vec<String>>,   // author -> chunk_ids
}

#[derive(Debug, Clone)]
struct RankedResult {
    chunk_id: String,
    score: f32,
    position: usize,
}

impl PartialEq for RankedResult {
    fn eq(&self, other: &Self) -> bool {
        (self.score - other.score).abs() < 0.001
    }
}

impl Eq for RankedResult {}

impl PartialOrd for RankedResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.score.partial_cmp(&self.score) // Reverse for max heap
    }
}

impl Ord for RankedResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl SearchEngine {
    pub fn new() -> Self {
        SearchEngine {
            chunks: HashMap::new(),
            inverted_index: HashMap::new(),
            clause_index: HashMap::new(),
            issue_index: HashMap::new(),
            collection_index: HashMap::new(),
            author_index: HashMap::new(),
        }
    }

    pub fn load_corpus_json(&mut self, json_str: &str) -> Result<String, String> {
        let corpus: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse corpus JSON: {}", e))?;

        let chunks_array = corpus.get("chunks")
            .and_then(|c| c.as_array())
            .ok_or("Corpus must have 'chunks' array")?;

        self.clear();

        for chunk_json in chunks_array {
            let chunk: Chunk = serde_json::from_value(chunk_json.clone())
                .map_err(|e| format!("Failed to parse chunk: {}", e))?;

            self.add_chunk(chunk);
        }

        Ok(format!("Loaded {} chunks", self.chunks.len()))
    }

    fn add_chunk(&mut self, chunk: Chunk) {
        // Extract collection from chunk_id (e.g., "constitution_0_0" -> "constitution")
        let collection = chunk.chunk_id.split('_').next().unwrap_or("unknown").to_string();

        // Index text tokens
        let tokens = tokenize(&chunk.text);
        for token in &tokens {
            let word = normalize(token);
            self.inverted_index
                .entry(word)
                .or_insert_with(Vec::new)
                .push(chunk.chunk_id.clone());
        }

        // Index clauses
        for clause in &chunk.constitutional_clause_tags {
            self.clause_index
                .entry(clause.clone())
                .or_insert_with(Vec::new)
                .push(chunk.chunk_id.clone());
        }

        // Index issues
        for issue in &chunk.issue_tags {
            self.issue_index
                .entry(issue.clone())
                .or_insert_with(Vec::new)
                .push(chunk.chunk_id.clone());
        }

        // Index collection
        self.collection_index
            .entry(collection)
            .or_insert_with(Vec::new)
            .push(chunk.chunk_id.clone());

        // Index author
        self.author_index
            .entry(chunk.author.clone())
            .or_insert_with(Vec::new)
            .push(chunk.chunk_id.clone());

        self.chunks.insert(chunk.chunk_id.clone(), chunk);
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let filters: HashMap<String, Vec<String>> = HashMap::new();
        self.search_with_filters(query, &filters, limit)
    }

    pub fn search_with_filters(
        &self,
        query: &str,
        filters: &HashMap<String, Vec<String>>,
        limit: usize,
    ) -> Vec<SearchResult> {
        if query.is_empty() {
            return Vec::new();
        }

        let query_tokens = tokenize(query);
        let mut candidate_chunks: Option<HashSet<String>> = None;

        // Find chunks containing all query terms (AND logic)
        for token in &query_tokens {
            let word = normalize(token);
            if let Some(chunk_ids) = self.inverted_index.get(&word) {
                let set: HashSet<String> = chunk_ids.iter().cloned().collect();
                candidate_chunks = match candidate_chunks {
                    None => Some(set),
                    Some(existing) => Some(existing.intersection(&set).cloned().collect()),
                };
            } else {
                return Vec::new(); // No results
            }
        }

        let mut candidates = candidate_chunks.unwrap_or_default();

        // Apply filters
        if let Some(collections) = filters.get("collections") {
            let mut filtered = HashSet::new();
            for collection in collections {
                if let Some(chunk_ids) = self.collection_index.get(collection) {
                    for id in chunk_ids {
                        if candidates.contains(id) {
                            filtered.insert(id.clone());
                        }
                    }
                }
            }
            candidates = filtered;
        }

        if let Some(clauses) = filters.get("clauses") {
            let mut filtered = HashSet::new();
            for clause in clauses {
                if let Some(chunk_ids) = self.clause_index.get(clause) {
                    for id in chunk_ids {
                        if candidates.contains(id) {
                            filtered.insert(id.clone());
                        }
                    }
                }
            }
            candidates = filtered;
        }

        if let Some(issues) = filters.get("issues") {
            let mut filtered = HashSet::new();
            for issue in issues {
                if let Some(chunk_ids) = self.issue_index.get(issue) {
                    for id in chunk_ids {
                        if candidates.contains(id) {
                            filtered.insert(id.clone());
                        }
                    }
                }
            }
            candidates = filtered;
        }

        if let Some(authors) = filters.get("authors") {
            let mut filtered = HashSet::new();
            for author in authors {
                if let Some(chunk_ids) = self.author_index.get(author) {
                    for id in chunk_ids {
                        if candidates.contains(id) {
                            filtered.insert(id.clone());
                        }
                    }
                }
            }
            candidates = filtered;
        }

        // Rank results
        let mut heap = BinaryHeap::new();

        for chunk_id in candidates {
            if let Some(chunk) = self.chunks.get(&chunk_id) {
                let score = self.calculate_relevance(&chunk.text, &query_tokens, chunk);
                let position = chunk.text.find(query).unwrap_or(0);

                heap.push(RankedResult {
                    chunk_id,
                    score,
                    position,
                });
            }
        }

        // Extract top results
        let mut results = Vec::new();
        while let Some(ranked) = heap.pop() {
            if results.len() >= limit {
                break;
            }

            if let Some(chunk) = self.chunks.get(&ranked.chunk_id) {
                let snippet = self.extract_snippet(&chunk.text, query, 100);
                results.push(SearchResult {
                    chunk_id: ranked.chunk_id.clone(),
                    title: chunk.title.clone(),
                    author: chunk.author.clone(),
                    text_snippet: snippet,
                    relevance_score: ranked.score,
                    position: ranked.position,
                });
            }
        }

        results
    }

    fn calculate_relevance(&self, text: &str, query_tokens: &[String], chunk: &Chunk) -> f32 {
        let text_lower = text.to_lowercase();
        let mut score = 0.0f32;

        for token in query_tokens {
            let word = normalize(token);
            let count = text_lower.matches(&word).count() as f32;
            score += count; // Term frequency
        }

        // Boost by clause relevance
        score += chunk.constitutional_clause_tags.len() as f32 * 2.0;

        // Boost by issue relevance
        score += chunk.issue_tags.len() as f32 * 1.5;

        // Normalize
        score / (query_tokens.len().max(1) as f32)
    }

    fn extract_snippet(&self, text: &str, query: &str, context: usize) -> String {
        if let Some(pos) = text.find(query) {
            let start = pos.saturating_sub(context);
            let end = (pos + query.len() + context).min(text.len());
            let snippet = &text[start..end];

            if start > 0 {
                format!("...{}", snippet)
            } else {
                snippet.to_string()
            }
        } else {
            text.chars()
                .take(context * 2)
                .collect::<String>() + "..."
        }
    }

    pub fn get_chunk(&self, chunk_id: &str) -> Option<Chunk> {
        self.chunks.get(chunk_id).cloned()
    }

    pub fn get_available_filters(&self) -> serde_json::Value {
        let collections: Vec<String> = self.collection_index.keys().cloned().collect();
        let clauses: Vec<String> = self.clause_index.keys().cloned().collect();
        let issues: Vec<String> = self.issue_index.keys().cloned().collect();
        let authors: Vec<String> = self.author_index.keys().cloned().collect();

        json!({
            "collections": collections,
            "clauses": clauses,
            "issues": issues,
            "authors": authors,
        })
    }

    pub fn get_corpus_stats(&self) -> CorpusStats {
        let total_words: usize = self.chunks.iter().map(|(_, c)| c.word_count).sum();
        let sizes: Vec<usize> = self.chunks.iter().map(|(_, c)| c.word_count).collect();

        let authors: HashSet<String> = self.chunks.iter().map(|(_, c)| c.author.clone()).collect();
        let collections: HashSet<String> = self.chunks.iter()
            .map(|(_, c)| c.chunk_id.split('_').next().unwrap_or("unknown").to_string())
            .collect();

        CorpusStats {
            total_chunks: self.chunks.len(),
            total_documents: self.chunks.iter().map(|(_, c)| c.document_id.clone()).collect::<HashSet<_>>().len(),
            total_words,
            average_chunk_size: if self.chunks.is_empty() { 0.0 } else { total_words as f32 / self.chunks.len() as f32 },
            min_chunk_size: sizes.iter().cloned().min().unwrap_or(0),
            max_chunk_size: sizes.iter().cloned().max().unwrap_or(0),
            unique_authors: authors.len(),
            unique_collections: collections.len(),
            unique_clauses: self.clause_index.len(),
            unique_issues: self.issue_index.len(),
        }
    }

    fn clear(&mut self) {
        self.chunks.clear();
        self.inverted_index.clear();
        self.clause_index.clear();
        self.issue_index.clear();
        self.collection_index.clear();
        self.author_index.clear();
    }
}
