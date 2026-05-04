//! Document chunking with multiple strategies
//!
//! Implements paragraph-aware chunking with configurable overlap,
//! and document-specific strategies for Constitution, Federalist Papers, etc.

use crate::error::Result;
use crate::types::{Chunk, ChunkId, ChunkStrategy, Document};
use regex::Regex;

/// Document chunker with configurable strategies
#[derive(Debug, Clone)]
pub struct Chunker {
    strategy: ChunkStrategy,
}

impl Default for Chunker {
    fn default() -> Self {
        Self::new(ChunkStrategy::default())
    }
}

impl Chunker {
    /// Create a chunker with specified strategy
    pub fn new(strategy: ChunkStrategy) -> Self {
        Self { strategy }
    }

    /// Chunk a document according to the configured strategy
    pub fn chunk(&self, document: &Document) -> Result<Vec<Chunk>> {
        match &self.strategy {
            ChunkStrategy::ConstitutionSections => self.chunk_constitution(document),
            ChunkStrategy::FederalistEssays => self.chunk_federalist(document),
            ChunkStrategy::JeffersonLetters => self.chunk_jefferson(document),
            ChunkStrategy::SlidingWindow {
                target_words,
                min_words,
                max_words,
                overlap_words,
            } => self.chunk_sliding_window(
                document,
                *target_words,
                *min_words,
                *max_words,
                *overlap_words,
            ),
        }
    }

    /// Chunk by Constitution article markers (Article I, II, III, etc.)
    fn chunk_constitution(&self, document: &Document) -> Result<Vec<Chunk>> {
        let article_regex = Regex::new(r"(?i)Article\s+([IVX]+)").unwrap();

        let mut chunks = Vec::new();
        let mut seq = 0;

        // First chunk: preamble
        if let Some(first_article) = article_regex.find(&document.text) {
            let preamble = document.text[..first_article.start()].trim();
            if !preamble.is_empty() {
                let chunk_id = ChunkId::from_doc_and_seq(&document.id, seq);
                let word_count = preamble.split_whitespace().count();
                let preview = Chunk::make_preview(preamble, 40);

                chunks.push(Chunk {
                    id: chunk_id,
                    document_id: document.id.clone(),
                    title: format!("{} - Preamble", document.title),
                    text: preamble.to_string(),
                    word_count,
                    preview,
                    issue_tags: document.metadata.get("issue_tags").cloned().into_iter().collect(),
                    clause_tags: document.metadata.get("clause_tags").cloned().into_iter().collect(),
                    metadata: document.metadata.clone(),
                });
                seq += 1;
            }
        }

        // Split by articles
        let mut last_end = 0;
        for caps in article_regex.captures_iter(&document.text) {
            let _article_num = caps.get(1).unwrap().as_str();
            let start = caps.get(0).unwrap().start();

            if start > last_end {
                let text = document.text[last_end..start].trim();
                if !text.is_empty() && seq > 1 {
                    let chunk_id = ChunkId::from_doc_and_seq(&document.id, seq);
                    let word_count = text.split_whitespace().count();
                    let preview = Chunk::make_preview(text, 40);

                    chunks.push(Chunk {
                        id: chunk_id,
                        document_id: document.id.clone(),
                        title: format!("{} - Part", document.title),
                        text: text.to_string(),
                        word_count,
                        preview,
                        issue_tags: vec![],
                        clause_tags: vec![],
                        metadata: document.metadata.clone(),
                    });
                    seq += 1;
                }
            }

            last_end = start;
        }

        // Final chunk
        if last_end < document.text.len() {
            let text = document.text[last_end..].trim();
            if !text.is_empty() {
                let chunk_id = ChunkId::from_doc_and_seq(&document.id, seq);
                let word_count = text.split_whitespace().count();
                let preview = Chunk::make_preview(text, 40);

                chunks.push(Chunk {
                    id: chunk_id,
                    document_id: document.id.clone(),
                    title: format!("{} - Final", document.title),
                    text: text.to_string(),
                    word_count,
                    preview,
                    issue_tags: vec![],
                    clause_tags: vec![],
                    metadata: document.metadata.clone(),
                });
            }
        }

        Ok(chunks)
    }

    /// Chunk Federalist Papers by "FEDERALIST No." headers
    fn chunk_federalist(&self, document: &Document) -> Result<Vec<Chunk>> {
        let essay_regex = Regex::new(r"(?i)FEDERALIST\s+(?:No\.?|Number)\s+(\d+)").unwrap();

        let mut chunks = Vec::new();
        let mut seq = 0;

        let mut last_end = 0;
        for caps in essay_regex.captures_iter(&document.text) {
            let essay_num = caps.get(1).unwrap().as_str();
            let start = caps.get(0).unwrap().start();

            // Extract previous essay (if any)
            if start > last_end && seq > 0 {
                let text = document.text[last_end..start].trim();
                if !text.is_empty() {
                    let chunk_id = ChunkId::from_doc_and_seq(&document.id, seq);
                    let word_count = text.split_whitespace().count();
                    let preview = Chunk::make_preview(text, 40);

                    chunks.push(Chunk {
                        id: chunk_id,
                        document_id: document.id.clone(),
                        title: format!("{} - Essay {}", document.title, essay_num),
                        text: text.to_string(),
                        word_count,
                        preview,
                        issue_tags: vec![],
                        clause_tags: vec![],
                        metadata: document.metadata.clone(),
                    });
                    seq += 1;
                }
            }

            last_end = start;
        }

        // Final essay
        if last_end < document.text.len() {
            let text = document.text[last_end..].trim();
            if !text.is_empty() {
                let chunk_id = ChunkId::from_doc_and_seq(&document.id, seq);
                let word_count = text.split_whitespace().count();
                let preview = Chunk::make_preview(text, 40);

                chunks.push(Chunk {
                    id: chunk_id,
                    document_id: document.id.clone(),
                    title: format!("{} - Essay", document.title),
                    text: text.to_string(),
                    word_count,
                    preview,
                    issue_tags: vec![],
                    clause_tags: vec![],
                    metadata: document.metadata.clone(),
                });
            }
        }

        Ok(chunks)
    }

    /// Chunk Jefferson letters by "TO {NAME}" patterns
    fn chunk_jefferson(&self, document: &Document) -> Result<Vec<Chunk>> {
        let letter_regex = Regex::new(r"(?i)^TO\s+([A-Z][A-Za-z\s]+)$").unwrap();

        let mut chunks = Vec::new();
        let mut seq = 0;

        // Split by paragraphs and look for TO patterns
        let paragraphs: Vec<&str> = document.text.split("\n\n").collect();

        let mut current_chunk = String::new();
        let mut recipient = "Unknown".to_string();

        for para in paragraphs {
            let trimmed = para.trim();

            if let Some(caps) = letter_regex.captures(trimmed) {
                // Save previous chunk if exists
                if !current_chunk.is_empty() {
                    let chunk_id = ChunkId::from_doc_and_seq(&document.id, seq);
                    let word_count = current_chunk.split_whitespace().count();
                    if word_count >= 80 {
                        let preview = Chunk::make_preview(&current_chunk, 40);
                        chunks.push(Chunk {
                            id: chunk_id,
                            document_id: document.id.clone(),
                            title: format!("{} - To {}", document.title, recipient),
                            text: current_chunk.clone(),
                            word_count,
                            preview,
                            issue_tags: vec![],
                            clause_tags: vec![],
                            metadata: document.metadata.clone(),
                        });
                        seq += 1;
                    }
                    current_chunk.clear();
                }

                // Start new letter
                recipient = caps.get(1).unwrap().as_str().trim().to_string();
                current_chunk = format!("TO {}\n\n", recipient);
            } else {
                current_chunk.push_str(trimmed);
                current_chunk.push_str("\n\n");
            }
        }

        // Final chunk
        if !current_chunk.is_empty() {
            let chunk_id = ChunkId::from_doc_and_seq(&document.id, seq);
            let word_count = current_chunk.split_whitespace().count();
            let preview = Chunk::make_preview(&current_chunk, 40);
            chunks.push(Chunk {
                id: chunk_id,
                document_id: document.id.clone(),
                title: format!("{} - To {}", document.title, recipient),
                text: current_chunk,
                word_count,
                preview,
                issue_tags: vec![],
                clause_tags: vec![],
                metadata: document.metadata.clone(),
            });
        }

        Ok(chunks)
    }

    /// Generic sliding window chunking with overlap
    fn chunk_sliding_window(
        &self,
        document: &Document,
        target_words: usize,
        min_words: usize,
        max_words: usize,
        overlap_words: usize,
    ) -> Result<Vec<Chunk>> {
        let words: Vec<&str> = document.text.split_whitespace().collect();
        let mut chunks = Vec::new();
        let mut seq = 0;

        let mut pos = 0;
        while pos < words.len() {
            let end = (pos + target_words).min(words.len());
            let actual_count = end - pos;

            // Adjust chunk size if needed
            let chunk_size = if actual_count < min_words {
                words.len() - pos // Take remaining
            } else if actual_count > max_words {
                max_words
            } else {
                actual_count
            };

            let chunk_end = (pos + chunk_size).min(words.len());
            let chunk_text = words[pos..chunk_end].join(" ");

            if chunk_text.split_whitespace().count() >= min_words {
                let chunk_id = ChunkId::from_doc_and_seq(&document.id, seq);
                let word_count = chunk_text.split_whitespace().count();
                let preview = Chunk::make_preview(&chunk_text, 40);

                chunks.push(Chunk {
                    id: chunk_id,
                    document_id: document.id.clone(),
                    title: document.title.clone(),
                    text: chunk_text,
                    word_count,
                    preview,
                    issue_tags: vec![],
                    clause_tags: vec![],
                    metadata: document.metadata.clone(),
                });
                seq += 1;
            }

            // Move position with overlap
            pos = chunk_end.saturating_sub(overlap_words).max(pos + 1);
        }

        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_doc(title: &str, text: &str) -> Document {
        Document {
            id: Default::default(),
            title: title.to_string(),
            author: None,
            date: None,
            source_collection: "test".to_string(),
            source_url: None,
            document_type: "test".to_string(),
            text: text.to_string(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_chunker_creation() {
        let chunker = Chunker::default();
        assert!(!format!("{:?}", chunker.strategy).is_empty());
    }

    #[test]
    fn test_sliding_window() {
        let chunker = Chunker::new(ChunkStrategy::SlidingWindow {
            target_words: 20,
            min_words: 10,
            max_words: 30,
            overlap_words: 5,
        });

        let doc = create_test_doc(
            "Test",
            &"word ".repeat(50),
        );

        let chunks = chunker.chunk(&doc).unwrap();
        assert!(!chunks.is_empty());
        assert!(chunks.iter().all(|c| c.word_count >= 10));
    }

    #[test]
    fn test_constitution_chunking() {
        let chunker = Chunker::new(ChunkStrategy::ConstitutionSections);
        let doc = create_test_doc(
            "Constitution",
            "Preamble text.\nArticle I\nLegislative content.\nArticle II\nExecutive content.",
        );

        let chunks = chunker.chunk(&doc).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_federalist_chunking() {
        let chunker = Chunker::new(ChunkStrategy::FederalistEssays);
        let doc = create_test_doc(
            "Federalist",
            "FEDERALIST No. 1\nContent one.\nFEDERALIST No. 2\nContent two.",
        );

        let chunks = chunker.chunk(&doc).unwrap();
        assert!(!chunks.is_empty());
    }
}

