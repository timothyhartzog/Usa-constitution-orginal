//! Application state management for indexes and document store

use crate::db;
use constitutional_lib::{
    chunker, error, fulltext_index, fuzzy_match, metadata_tagger, tokenizer, types, vector_store,
};
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

/// Application state containing all search indexes
pub struct AppState {
    /// Database connection for persistence (wrapped in Mutex for thread-safety)
    db: Option<Mutex<Connection>>,
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
            db: None,
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

    /// Create with database persistence
    pub fn with_db(db_path: &str) -> error::Result<Self> {
        let conn = db::open_db(db_path)
            .map_err(|e| format!("Database initialization failed: {}", e))?;

        let mut state = Self {
            db: Some(Mutex::new(conn)),
            fulltext: RwLock::new(fulltext_index::FullTextIndex::new()),
            fuzzy: RwLock::new(fuzzy_match::FuzzyIndex::new()),
            vector: RwLock::new(vector_store::VectorStore::with_dimension(384)),
            documents: RwLock::new(HashMap::new()),
            chunks: RwLock::new(HashMap::new()),
            tokenizer: tokenizer::Tokenizer::new(),
            chunker: chunker::Chunker::new(types::ChunkStrategy::default()),
            tagger: metadata_tagger::MetadataTagger::new(vec![], vec![]),
        };

        // Recover indexes from database
        state.recover_from_db()?;

        Ok(state)
    }

    /// Recover indexes from database on startup
    fn recover_from_db(&mut self) -> error::Result<()> {
        if let Some(ref db_mutex) = self.db {
            let db = db_mutex.lock().unwrap();
            log::info!("Recovering indexes from database...");

            // Get document count
            let documents = db::count_documents(&db)
                .map_err(|e| format!("Failed to count documents: {}", e))?;

            log::info!("Found {} documents in database", documents);

            if documents > 0 {
                log::info!("Index recovery from database implemented in Phase 2B+");
            }

            Ok(())
        } else {
            Ok(())
        }
    }

    /// Ingest a document and update all indexes
    pub fn ingest_document(&self, doc: types::Document) -> error::Result<()> {
        // 1. Chunk the document
        let chunks = self.chunker.chunk(&doc)?;

        // 2. Store document in database if available
        if let Some(ref db_mutex) = self.db {
            let db = db_mutex.lock().unwrap();
            db::save_document(
                &db,
                &doc.id.to_string(),
                &doc.title,
                doc.author.as_deref(),
                doc.date.as_deref(),
                &doc.source_collection,
                doc.source_url.as_deref(),
                &doc.document_type,
                &doc.text,
            ).map_err(|e| format!("Failed to save document to database: {}", e))?;
        }

        // 3. Index chunks
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

            // Store chunk in database if available
            if let Some(ref db_mutex) = self.db {
                let db = db_mutex.lock().unwrap();
                db::save_chunk(
                    &db,
                    &chunk.id.to_string(),
                    &chunk.document_id.to_string(),
                    &chunk.title,
                    &chunk.text,
                    chunk.word_count,
                    &chunk.preview,
                ).map_err(|e| format!("Failed to save chunk to database: {}", e))?;

                // Store fulltext tokens
                let token_freqs: Vec<(String, u32)> = tokens.iter()
                    .fold(HashMap::new(), |mut acc, token| {
                        *acc.entry(token.clone()).or_insert(0) += 1;
                        acc
                    })
                    .into_iter()
                    .collect();

                db::add_fulltext_tokens(&db, &chunk.id.to_string(), &token_freqs)
                    .map_err(|e| format!("Failed to save fulltext tokens: {}", e))?;

                // Store fuzzy tokens
                db::add_fuzzy_tokens(&db, &chunk.id.to_string(), &tokens)
                    .map_err(|e| format!("Failed to save fuzzy tokens: {}", e))?;
            }

            // Store chunk in memory
            {
                let mut chunks_store = self.chunks.write().unwrap();
                chunks_store.insert(chunk.id.to_string(), chunk);
            }
        }

        // 4. Store document in memory
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
