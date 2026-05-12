use std::collections::BTreeSet;
use std::fs;

use constitution_archive::Chunk;
use serde::{Deserialize, Serialize};

use crate::paths::PipelinePaths;

#[derive(Debug, Deserialize)]
struct CorpusFile {
    chunks: Vec<Chunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorpusSummary {
    pub chunks: usize,
    pub documents: usize,
    pub collections: usize,
    pub authors: usize,
    pub empty_text_chunks: usize,
    pub duplicate_chunk_ids: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationReport {
    pub corpus: Option<CorpusSummary>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    pub fn is_clean(&self) -> bool {
        self.warnings.is_empty()
    }
}

pub fn validate_corpus(paths: &PipelinePaths) -> anyhow::Result<ValidationReport> {
    if !paths.corpus_json.exists() {
        return Ok(ValidationReport {
            corpus: None,
            warnings: vec![format!(
                "missing canonical corpus JSON: {}",
                paths.corpus_json.display()
            )],
        });
    }

    let bytes = fs::read(&paths.corpus_json)?;
    let corpus: CorpusFile = serde_json::from_slice(&bytes)?;
    let mut document_ids = BTreeSet::new();
    let mut collections = BTreeSet::new();
    let mut authors = BTreeSet::new();
    let mut seen_chunks = BTreeSet::new();
    let mut empty_text_chunks = 0;
    let mut duplicate_chunk_ids = 0;
    let mut warnings = Vec::new();

    for chunk in &corpus.chunks {
        if chunk.text.trim().is_empty() {
            empty_text_chunks += 1;
        }
        if !seen_chunks.insert(chunk.chunk_id.clone()) {
            duplicate_chunk_ids += 1;
        }
        if chunk.document_id.trim().is_empty() {
            warnings.push(format!("chunk {} has no document_id", chunk.chunk_id));
        }
        document_ids.insert(chunk.document_id.clone());
        collections.insert(chunk.source_collection.clone());
        authors.insert(chunk.author.clone());
    }

    if empty_text_chunks > 0 {
        warnings.push(format!("{empty_text_chunks} chunks have empty text"));
    }
    if duplicate_chunk_ids > 0 {
        warnings.push(format!("{duplicate_chunk_ids} duplicate chunk ids"));
    }

    Ok(ValidationReport {
        corpus: Some(CorpusSummary {
            chunks: corpus.chunks.len(),
            documents: document_ids.len(),
            collections: collections.len(),
            authors: authors.len(),
            empty_text_chunks,
            duplicate_chunk_ids,
        }),
        warnings,
    })
}
