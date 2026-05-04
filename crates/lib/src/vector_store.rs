//! Vector store for semantic search using embeddings
//!
//! Implements dense vector storage with cosine similarity search
//! for semantic/conceptual matching of documents.

use crate::error::Result;
use crate::types::ChunkId;
use std::collections::HashMap;

/// Vector store for semantic search with embeddings
#[derive(Debug, Clone)]
pub struct VectorStore {
    embeddings: HashMap<ChunkId, Vec<f32>>,
    dimension: usize,
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorStore {
    /// Create a new vector store
    pub fn new() -> Self {
        Self {
            embeddings: HashMap::new(),
            dimension: 0,
        }
    }

    /// Create a vector store with specified embedding dimension
    pub fn with_dimension(dimension: usize) -> Self {
        Self {
            embeddings: HashMap::new(),
            dimension,
        }
    }

    /// Add an embedding for a chunk
    pub fn add_embedding(&mut self, chunk_id: ChunkId, embedding: Vec<f32>) -> Result<()> {
        if embedding.is_empty() {
            return Err("Embedding cannot be empty".into());
        }

        // Set dimension on first embedding
        if self.dimension == 0 {
            self.dimension = embedding.len();
        } else if embedding.len() != self.dimension {
            return Err(format!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding.len()
            )
            .into());
        }

        // Validate embedding is normalized or normalize it
        let normalized = normalize_vector(&embedding);
        self.embeddings.insert(chunk_id, normalized);

        Ok(())
    }

    /// Search for similar chunks using cosine similarity
    pub fn search(&self, query_embedding: &[f32], k: usize) -> Result<Vec<(ChunkId, f32)>> {
        if query_embedding.is_empty() {
            return Err("Query embedding cannot be empty".into());
        }

        if query_embedding.len() != self.dimension && self.dimension > 0 {
            return Err(format!(
                "Query embedding dimension mismatch: expected {}, got {}",
                self.dimension,
                query_embedding.len()
            )
            .into());
        }

        let query_norm = normalize_vector(query_embedding);
        let mut results = Vec::new();

        // Compute cosine similarity for all stored embeddings
        for (chunk_id, embedding) in &self.embeddings {
            let similarity = cosine_similarity(&query_norm, embedding);
            results.push((chunk_id.clone(), similarity));
        }

        // Sort by similarity descending and take top k
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(k);

        Ok(results)
    }

    /// Get number of embeddings in store
    pub fn len(&self) -> usize {
        self.embeddings.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.embeddings.is_empty()
    }

    /// Get the embedding dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Clear the store
    pub fn clear(&mut self) {
        self.embeddings.clear();
        self.dimension = 0;
    }

    /// Get a reference to an embedding
    pub fn get(&self, chunk_id: &ChunkId) -> Option<&Vec<f32>> {
        self.embeddings.get(chunk_id)
    }
}

/// Normalize a vector to unit length (L2 normalization)
fn normalize_vector(vec: &[f32]) -> Vec<f32> {
    let magnitude = (vec.iter().map(|v| v * v).sum::<f32>()).sqrt();

    if magnitude == 0.0 {
        vec.to_vec()
    } else {
        vec.iter().map(|v| v / magnitude).collect()
    }
}

/// Calculate cosine similarity between two normalized vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_store_creation() {
        let store = VectorStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_vector_store_with_dimension() {
        let store = VectorStore::with_dimension(384);
        assert_eq!(store.dimension(), 384);
    }

    #[test]
    fn test_add_embedding() {
        let mut store = VectorStore::new();
        let chunk_id = ChunkId::new("doc1_0000");
        let embedding = vec![0.1, 0.2, 0.3];

        assert!(store.add_embedding(chunk_id.clone(), embedding).is_ok());
        assert_eq!(store.len(), 1);
        assert_eq!(store.dimension(), 3);
    }

    #[test]
    fn test_dimension_mismatch() {
        let mut store = VectorStore::new();
        let chunk1 = ChunkId::new("doc1_0000");
        let chunk2 = ChunkId::new("doc2_0000");

        store
            .add_embedding(chunk1, vec![0.1, 0.2, 0.3])
            .unwrap();

        // Different dimension should fail
        let result = store.add_embedding(chunk2, vec![0.1, 0.2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&vec1, &vec2);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let vec1 = vec![1.0, 0.0];
        let vec2 = vec![0.0, 1.0];
        let sim = cosine_similarity(&vec1, &vec2);
        assert!(sim.abs() < 0.001);
    }

    #[test]
    fn test_search() {
        let mut store = VectorStore::new();
        let chunk1 = ChunkId::new("doc1_0000");
        let chunk2 = ChunkId::new("doc2_0000");

        store
            .add_embedding(chunk1.clone(), vec![1.0, 0.0, 0.0])
            .unwrap();
        store
            .add_embedding(chunk2.clone(), vec![0.0, 1.0, 0.0])
            .unwrap();

        let query = vec![1.0, 0.0, 0.0];
        let results = store.search(&query, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, chunk1); // Most similar
    }

    #[test]
    fn test_search_top_k() {
        let mut store = VectorStore::new();

        for i in 0..5 {
            let mut embedding = vec![0.0; 10];
            embedding[0] = (i as f32) / 5.0;
            store
                .add_embedding(
                    ChunkId::new(format!("doc{}_0000", i)),
                    embedding,
                )
                .unwrap();
        }

        let query = vec![1.0; 10];
        let results = store.search(&query, 2).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_normalize_vector() {
        let vec = vec![3.0, 4.0];
        let normalized = normalize_vector(&vec);
        let magnitude = (normalized.iter().map(|v| v * v).sum::<f32>()).sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_get_embedding() {
        let mut store = VectorStore::new();
        let chunk_id = ChunkId::new("doc1_0000");
        let embedding = vec![0.1, 0.2, 0.3];

        store
            .add_embedding(chunk_id.clone(), embedding.clone())
            .unwrap();

        let retrieved = store.get(&chunk_id).unwrap();
        // Embedding is normalized when stored, so compare magnitudes instead
        let normalized = normalize_vector(&embedding);
        assert!((retrieved[0] - normalized[0]).abs() < 0.001);
    }
}

