//! Application state management for indexes and document store

use constitutional_lib::{
    chunker, error, fulltext_index, fuzzy_match, metadata_tagger, tokenizer, types, vector_store,
};
use std::collections::HashMap;
use std::sync::RwLock;

/// Application state containing all search indexes
pub struct AppState {
    /// Full-text search index
    fulltext: RwLock<fulltext_index::FullTextIndex>,
    /// Fuzzy search index (BK-tree)
    fuzzy: RwLock<fuzzy_match::FuzzyIndex>,
    /// Vector store for semantic search
    vector: RwLock<vector_store::VectorStore>,
    /// Document store
    documents: RwLock<HashMap<String, types::Document>>,
    /// Chunk store
    chunks: RwLock<HashMap<String, types::Chunk>>,
    /// Tokenizer for processing text
    tokenizer: tokenizer::Tokenizer,
    /// Chunker for splitting documents
    chunker: chunker::Chunker,
    /// Metadata tagger
    tagger: metadata_tagger::MetadataTagger,
}

impl AppState {
    /// Create new application state with empty indexes
    pub fn new() -> Self {
        Self {
            fulltext: RwLock::new(fulltext_index::FullTextIndex::new()),
            fuzzy: RwLock::new(fuzzy_match::FuzzyIndex::new()),
            vector: RwLock::new(vector_store::VectorStore::with_dimension(384)),
            documents: RwLock::new(HashMap::new()),
            chunks: RwLock::new(HashMap::new()),
            tokenizer: tokenizer::Tokenizer::new(),
            chunker: chunker::Chunker::new(types::ChunkStrategy::default()),
            tagger: metadata_tagger::MetadataTagger::new(vec![], vec![]),
        }
    }

    /// Ingest a document and update all indexes
    pub fn ingest_document(&self, mut doc: types::Document) -> error::Result<()> {
        // 1. Chunk the document
        let chunks = self.chunker.chunk(&doc)?;

        // 2. Index chunks
        for chunk in chunks {
            // Tokenize chunk text
            let tokens = self.tokenizer.tokenize(&chunk.text)?;

            // Add to full-text index
            {
                let mut ft = self.fulltext.write().unwrap();
                ft.add_chunk(chunk.id.clone(), tokens.clone())?;
            }

            // Add to fuzzy index
            {
                let mut fz = self.fuzzy.write().unwrap();
                for token in &tokens {
                    fz.add_token(token.clone(), chunk.id.clone())?;
                }
            }

            // Store chunk
            {
                let mut chunks_store = self.chunks.write().unwrap();
                chunks_store.insert(chunk.id.to_string(), chunk);
            }
        }

        // 3. Store document
        {
            let mut docs = self.documents.write().unwrap();
            docs.insert(doc.id.to_string(), doc);
        }

        Ok(())
    }

    /// Get a chunk by ID
    pub fn get_chunk(&self, chunk_id: &str) -> Option<types::Chunk> {
        self.chunks
            .read()
            .unwrap()
            .get(chunk_id)
            .map(|c| c.clone())
    }

    /// Get a document by ID
    pub fn get_document(&self, doc_id: &str) -> Option<types::Document> {
        self.documents
            .read()
            .unwrap()
            .get(doc_id)
            .map(|d| d.clone())
    }

    /// Full-text search
    pub fn search_fulltext(&self, query: &str) -> error::Result<Vec<types::SearchResult>> {
        let tokens = self.tokenizer.tokenize(query)?;
        let ft = self.fulltext.read().unwrap();
        let results = ft.search(&tokens)?;

        let chunks_store = self.chunks.read().unwrap();
        let search_results: Vec<_> = results
            .into_iter()
            .filter_map(|(chunk_id, score)| {
                chunks_store.get(chunk_id.as_ref()).map(|chunk| {
                    types::SearchResult {
                        chunk: chunk.clone(),
                        score,
                        search_type: types::SearchType::FullText,
                    }
                })
            })
            .collect();

        Ok(search_results)
    }

    /// Fuzzy search
    pub fn search_fuzzy(&self, query: &str, max_distance: usize) -> error::Result<Vec<types::SearchResult>> {
        let fz = self.fuzzy.read().unwrap();
        let results = fz.search(query, max_distance)?;

        let chunks_store = self.chunks.read().unwrap();
        let search_results: Vec<_> = results
            .into_iter()
            .filter_map(|(chunk_id, score)| {
                chunks_store.get(chunk_id.as_ref()).map(|chunk| {
                    types::SearchResult {
                        chunk: chunk.clone(),
                        score,
                        search_type: types::SearchType::Fuzzy,
                    }
                })
            })
            .collect();

        Ok(search_results)
    }

    /// Get index statistics
    pub fn get_stats(&self) -> types::IndexStats {
        let documents = self.documents.read().unwrap();
        let chunks = self.chunks.read().unwrap();
        let ft = self.fulltext.read().unwrap();
        let (total_terms, _, _, _) = ft.stats();

        let total_chunks = chunks.len();
        let avg_chunk_size = if total_chunks > 0 {
            chunks.values().map(|c| c.word_count).sum::<usize>() as f32 / total_chunks as f32
        } else {
            0.0
        };

        types::IndexStats {
            total_documents: documents.len(),
            total_chunks,
            total_terms,
            avg_chunk_size,
            index_size_bytes: 0, // TODO: Calculate actual size
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
