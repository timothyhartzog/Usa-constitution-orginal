//! Fuzzy matching with BK-tree (Burkhard-Keller tree) for sub-linear search
//!
//! Implements metric space indexing on Levenshtein distance for
//! typo-tolerant search with O(log n) average case performance.

use crate::error::Result;
use crate::types::ChunkId;
use std::collections::HashMap;
use strsim::levenshtein;

/// Node in the BK-tree
#[derive(Debug, Clone)]
struct BKNode {
    token: String,
    chunk_id: ChunkId,
    children: HashMap<usize, BKNode>,
}

impl BKNode {
    fn new(token: String, chunk_id: ChunkId) -> Self {
        Self {
            token,
            chunk_id,
            children: HashMap::new(),
        }
    }

    fn to_json(&self) -> serde_json::Value {
        let mut children_json = serde_json::Map::new();
        for (dist, child) in &self.children {
            children_json.insert(dist.to_string(), child.to_json());
        }
        serde_json::json!({
            "token": self.token,
            "chunk_id": self.chunk_id.as_ref(),
            "children": children_json
        })
    }
}

/// Fuzzy search index using BK-tree for sub-linear search
#[derive(Debug, Clone)]
pub struct FuzzyIndex {
    root: Option<Box<BKNode>>,
    size: usize,
}

impl Default for FuzzyIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyIndex {
    /// Create a new fuzzy index
    pub fn new() -> Self {
        Self { root: None, size: 0 }
    }

    /// Add a token to the fuzzy index
    pub fn add_token(&mut self, token: String, chunk_id: ChunkId) -> Result<()> {
        if token.is_empty() {
            return Ok(());
        }

        if let Some(ref mut root) = self.root {
            insert_recursive(root, token, chunk_id);
        } else {
            self.root = Some(Box::new(BKNode::new(token, chunk_id)));
        }

        self.size += 1;
        Ok(())
    }

    /// Search for tokens matching query with edit distance tolerance
    pub fn search(&self, query: &str, max_distance: usize) -> Result<Vec<(ChunkId, f32)>> {
        let mut results = Vec::new();

        if let Some(root) = &self.root {
            self.search_recursive(root, query, max_distance, &mut results);
        }

        // Sort by relevance (lower distance = higher score)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(results)
    }

    /// Recursively search the BK-tree
    fn search_recursive(
        &self,
        node: &BKNode,
        query: &str,
        max_distance: usize,
        results: &mut Vec<(ChunkId, f32)>,
    ) {
        let distance = levenshtein(&node.token, query);

        // Check if this node matches
        if distance <= max_distance {
            // Score: 1.0 for exact match, decreases with distance
            let score = 1.0 - (distance as f32 / max_distance.max(1) as f32);
            results.push((node.chunk_id.clone(), score));
        }

        // Triangle inequality: search children within distance bounds
        for (&child_distance, child) in &node.children {
            let lower = distance.saturating_sub(max_distance);
            let upper = distance + max_distance;

            if child_distance >= lower && child_distance <= upper {
                self.search_recursive(child, query, max_distance, results);
            }
        }
    }

    /// Get the number of tokens in the index
    pub fn len(&self) -> usize {
        self.size
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.root = None;
        self.size = 0;
    }

    /// Export the index to a JSON representation
    pub fn export_json(&self) -> serde_json::Value {
        serde_json::json!({
            "size": self.size,
            "root": self.root.as_ref().map(|n| n.to_json())
        })
    }
}

/// Recursively insert a token into the BK-tree
fn insert_recursive(node: &mut BKNode, token: String, chunk_id: ChunkId) {
    let distance = levenshtein(&node.token, &token);

    if distance == 0 {
        // Duplicate token, skip
        return;
    }

    if let Some(child) = node.children.get_mut(&distance) {
        // Recursively insert into child
        insert_recursive(child, token, chunk_id);
    } else {
        // Create new leaf node
        node.children
            .insert(distance, BKNode::new(token, chunk_id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_index_creation() {
        let index = FuzzyIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_add_token() {
        let mut index = FuzzyIndex::new();
        let chunk_id = ChunkId::new("doc1_0000");
        index.add_token("constitution".to_string(), chunk_id).unwrap();
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_exact_match() {
        let mut index = FuzzyIndex::new();
        let chunk_id = ChunkId::new("doc1_0000");
        index
            .add_token("constitution".to_string(), chunk_id.clone())
            .unwrap();

        let results = index.search("constitution", 2).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].0, chunk_id);
        assert_eq!(results[0].1, 1.0); // Exact match
    }

    #[test]
    fn test_fuzzy_match_typo() {
        let mut index = FuzzyIndex::new();
        let chunk_id = ChunkId::new("doc1_0000");
        index
            .add_token("constitution".to_string(), chunk_id.clone())
            .unwrap();

        // One character typo
        let results = index.search("constittion", 2).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].0, chunk_id);
        assert!(results[0].1 > 0.0); // Fuzzy match
    }

    #[test]
    fn test_multiple_tokens() {
        let mut index = FuzzyIndex::new();
        let chunk1 = ChunkId::new("doc1_0000");
        let chunk2 = ChunkId::new("doc2_0000");

        index
            .add_token("constitution".to_string(), chunk1.clone())
            .unwrap();
        index
            .add_token("ratified".to_string(), chunk2.clone())
            .unwrap();

        assert_eq!(index.len(), 2);

        let results = index.search("constitution", 1).unwrap();
        assert_eq!(results[0].0, chunk1);
    }

    #[test]
    fn test_no_match_outside_distance() {
        let mut index = FuzzyIndex::new();
        let chunk_id = ChunkId::new("doc1_0000");
        index
            .add_token("constitution".to_string(), chunk_id)
            .unwrap();

        // Very different word, large distance
        let results = index.search("xyz", 1).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut index = FuzzyIndex::new();
        index
            .add_token("constitution".to_string(), ChunkId::new("doc1_0000"))
            .unwrap();
        assert!(!index.is_empty());

        index.clear();
        assert!(index.is_empty());
    }
}

