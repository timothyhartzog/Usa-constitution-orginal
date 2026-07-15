use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::MlError;
use crate::ChunkId;
use candle_core::{Device, Tensor};

/// A flat memory-mapped or loaded vector index for fast semantic search.
pub struct VectorIndex {
    /// Flattened float32 array
    embeddings: Tensor,
    /// Dimensions per vector
    dim: usize,
    /// Ordered chunk IDs corresponding to rows in `embeddings`
    chunk_ids: Vec<ChunkId>,
}

#[derive(Debug, Clone)]
pub struct VectorResult {
    pub chunk_id: ChunkId,
    pub score: f32,
}

impl VectorIndex {
    /// Load the vector index from the flat `.bin` and `.json` mapping created by Python.
    pub fn load<P1: AsRef<Path>, P2: AsRef<Path>>(
        bin_path: P1,
        mapping_path: P2,
        device: &Device,
    ) -> Result<Self, MlError> {
        let mut mapping_file =
            File::open(mapping_path).map_err(|e| MlError::Vector(e.to_string()))?;
        let mut json_str = String::new();
        mapping_file
            .read_to_string(&mut json_str)
            .map_err(|e| MlError::Vector(e.to_string()))?;

        let mapping: serde_json::Value =
            serde_json::from_str(&json_str).map_err(|e| MlError::Vector(e.to_string()))?;
        let chunk_ids: Vec<ChunkId> =
            serde_json::from_value(mapping["chunk_ids"].clone()).unwrap_or_default();
        let dim = mapping["dim"].as_u64().unwrap_or(768) as usize;

        let num_vecs = chunk_ids.len();

        let mut bin_file = File::open(bin_path).map_err(|e| MlError::Vector(e.to_string()))?;
        let mut raw_bytes = Vec::new();
        bin_file
            .read_to_end(&mut raw_bytes)
            .map_err(|e| MlError::Vector(e.to_string()))?;

        // Ensure size matches
        if raw_bytes.len() != num_vecs * dim * 4 {
            return Err(MlError::Vector("Binary vector data size mismatch".into()));
        }

        // Convert u8 back to f32
        let mut f32_data = vec![0.0f32; num_vecs * dim];
        for (i, chunk) in raw_bytes.chunks_exact(4).enumerate() {
            f32_data[i] = f32::from_le_bytes(chunk.try_into().unwrap());
        }

        let embeddings = Tensor::from_vec(f32_data, (num_vecs, dim), device)?;

        Ok(Self {
            embeddings,
            dim,
            chunk_ids,
        })
    }

    /// Run inner product (cosine similarity if normalized) against a query vector
    pub fn search(&self, query_vec: &Tensor, top_k: usize) -> Result<Vec<VectorResult>, MlError> {
        // query_vec shape: (1, dim)
        // embeddings shape: (N, dim)
        // scores shape: (1, N)
        let scores = query_vec.matmul(&self.embeddings.t()?)?;

        // Extract 1D array of scores
        let scores_1d = scores.squeeze(0)?.to_vec1::<f32>()?;

        // Sort and get top K
        let mut scored_indices: Vec<(usize, f32)> = scores_1d.into_iter().enumerate().collect();
        scored_indices.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut results = Vec::with_capacity(top_k);
        for (idx, score) in scored_indices.into_iter().take(top_k) {
            results.push(VectorResult {
                chunk_id: self.chunk_ids[idx].clone(),
                score,
            });
        }

        Ok(results)
    }
}
