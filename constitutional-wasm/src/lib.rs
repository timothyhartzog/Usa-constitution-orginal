use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod search;
mod analytics;
mod utils;

pub use search::SearchEngine;
pub use analytics::Analytics;

// Export for JavaScript
pub mod js {
    use super::*;

    #[wasm_bindgen]
    pub struct WasmSearchEngine {
        engine: SearchEngine,
    }

    #[wasm_bindgen]
    impl WasmSearchEngine {
        #[wasm_bindgen(constructor)]
        pub fn new() -> WasmSearchEngine {
            WasmSearchEngine {
                engine: SearchEngine::new(),
            }
        }

        #[wasm_bindgen]
        pub fn load_corpus(&mut self, corpus_json: &str) -> Result<String, JsValue> {
            self.engine.load_corpus_json(corpus_json)
                .map_err(|e| JsValue::from_str(&e))
        }

        #[wasm_bindgen]
        pub fn search(&self, query: &str, limit: usize) -> String {
            let results = self.engine.search(query, limit);
            serde_json::to_string(&results).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn search_with_filters(&self, query: &str, filters: &str, limit: usize) -> String {
            let filters: HashMap<String, Vec<String>> = serde_json::from_str(filters).unwrap_or_default();
            let results = self.engine.search_with_filters(query, &filters, limit);
            serde_json::to_string(&results).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn get_chunk(&self, chunk_id: &str) -> String {
            match self.engine.get_chunk(chunk_id) {
                Some(chunk) => serde_json::to_string(&chunk).unwrap_or_default(),
                None => "{}".to_string(),
            }
        }

        #[wasm_bindgen]
        pub fn get_filters(&self) -> String {
            let filters = self.engine.get_available_filters();
            serde_json::to_string(&filters).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn get_stats(&self) -> String {
            let stats = self.engine.get_corpus_stats();
            serde_json::to_string(&stats).unwrap_or_default()
        }
    }

    #[wasm_bindgen]
    pub struct WasmAnalytics {
        analytics: Analytics,
    }

    #[wasm_bindgen]
    impl WasmAnalytics {
        #[wasm_bindgen(constructor)]
        pub fn new() -> WasmAnalytics {
            WasmAnalytics {
                analytics: Analytics::new(),
            }
        }

        #[wasm_bindgen]
        pub fn load_corpus(&mut self, corpus_json: &str) -> Result<String, JsValue> {
            self.analytics.load_corpus(corpus_json)
                .map_err(|e| JsValue::from_str(&e))
        }

        #[wasm_bindgen]
        pub fn analyze_overview(&self) -> String {
            let overview = self.analytics.analyze_overview();
            serde_json::to_string(&overview).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn analyze_clauses(&self, top_n: usize) -> String {
            let clauses = self.analytics.analyze_clauses(top_n);
            serde_json::to_string(&clauses).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn analyze_issues(&self, top_n: usize) -> String {
            let issues = self.analytics.analyze_issues(top_n);
            serde_json::to_string(&issues).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn analyze_authors(&self) -> String {
            let authors = self.analytics.analyze_authors();
            serde_json::to_string(&authors).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn analyze_collections(&self) -> String {
            let collections = self.analytics.analyze_collections();
            serde_json::to_string(&collections).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn analyze_word_frequency(&self, top_n: usize) -> String {
            let words = self.analytics.analyze_word_frequency(top_n);
            serde_json::to_string(&words).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn analyze_clause_issue_matrix(&self) -> String {
            let matrix = self.analytics.analyze_clause_issue_matrix();
            serde_json::to_string(&matrix).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn analyze_author_clause_matrix(&self) -> String {
            let matrix = self.analytics.analyze_author_clause_matrix();
            serde_json::to_string(&matrix).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn filter_chunks(&self, filters: &str) -> String {
            let filters: HashMap<String, Vec<String>> = serde_json::from_str(filters).unwrap_or_default();
            let chunks = self.analytics.filter_chunks(&filters);
            serde_json::to_string(&chunks).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn compare_authors(&self, author1: &str, author2: &str) -> String {
            let comparison = self.analytics.compare_authors(author1, author2);
            serde_json::to_string(&comparison).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn analyze_temporal_network(&self) -> String {
            let network = self.analytics.analyze_temporal_network();
            serde_json::to_string(&network).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn analyze_clause_debate_network(&self) -> String {
            let network = self.analytics.analyze_clause_debate_network();
            serde_json::to_string(&network).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn analyze_author_influence(&self) -> String {
            let influence = self.analytics.analyze_author_influence();
            serde_json::to_string(&influence).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn analyze_semantic_similarity(&self) -> String {
            let clusters = self.analytics.analyze_semantic_similarity_clusters();
            serde_json::to_string(&clusters).unwrap_or_default()
        }

        #[wasm_bindgen]
        pub fn analyze_ratification(&self) -> String {
            let ratification = self.analytics.analyze_ratification_tracking();
            serde_json::to_string(&ratification).unwrap_or_default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub chunk_id: String,
    pub document_id: String,
    pub title: String,
    pub author: String,
    pub date: String,
    pub text: String,
    pub source_url: String,
    pub constitutional_clause_tags: Vec<String>,
    pub issue_tags: Vec<String>,
    pub word_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk_id: String,
    pub title: String,
    pub author: String,
    pub text_snippet: String,
    pub relevance_score: f32,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusStats {
    pub total_chunks: usize,
    pub total_documents: usize,
    pub total_words: usize,
    pub average_chunk_size: f32,
    pub min_chunk_size: usize,
    pub max_chunk_size: usize,
    pub unique_authors: usize,
    pub unique_collections: usize,
    pub unique_clauses: usize,
    pub unique_issues: usize,
}
