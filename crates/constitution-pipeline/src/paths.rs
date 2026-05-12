use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Stable locations used by the Rust pipeline.
///
/// These paths deliberately preserve the existing Python-era artifact layout so
/// the migration can happen one stage at a time without risking downloaded text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelinePaths {
    pub root: PathBuf,
    pub raw_dir: PathBuf,
    pub clean_dir: PathBuf,
    pub chunks_dir: PathBuf,
    pub index_dir: PathBuf,
    pub founders_online_dir: PathBuf,
    pub research_graph_dir: PathBuf,
    pub research_vectors_dir: PathBuf,
    pub pipeline_dir: PathBuf,
    pub corpus_json: PathBuf,
    pub archive_bin: PathBuf,
}

impl PipelinePaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let data = root.join("data");
        let chunks_dir = data.join("chunks");
        let index_dir = data.join("index");
        Self {
            raw_dir: data.join("raw"),
            clean_dir: data.join("clean"),
            chunks_dir: chunks_dir.clone(),
            index_dir: index_dir.clone(),
            founders_online_dir: data.join("founders_online"),
            research_graph_dir: data.join("research_graph"),
            research_vectors_dir: data.join("research_vectors"),
            pipeline_dir: data.join("pipeline"),
            corpus_json: chunks_dir.join("constitution_full_corpus.json"),
            archive_bin: index_dir.join("constitution_archive.bin"),
            root,
        }
    }

    pub fn display_from_root<'a>(&self, path: &'a Path) -> &'a Path {
        path.strip_prefix(&self.root).unwrap_or(path)
    }

    pub fn canonical_inputs(&self) -> [&Path; 5] {
        [
            self.raw_dir.as_path(),
            self.clean_dir.as_path(),
            self.chunks_dir.as_path(),
            self.founders_online_dir.as_path(),
            self.research_graph_dir.as_path(),
        ]
    }
}
