//! Server-shared state: an `Arc<Archive>` plus a path for static-file serving.

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use constitution_archive::graph::KnowledgeGraph;
#[cfg(feature = "ml")]
use constitution_archive::ml::RagEngine;
use constitution_archive::Archive;

/// Cheaply-cloneable handle to the loaded archive plus configuration.
#[derive(Clone)]
pub struct AppState {
    /// Loaded, queryable archive shared by every handler.
    pub archive: Arc<Archive>,
    /// Optional directory to expose at `/`. Typically the repository's
    /// `frontend/` directory so the WASM page can be served by the same
    /// process as the API.
    pub static_dir: Option<PathBuf>,
    /// The pre-compiled historical Knowledge Graph.
    pub knowledge_graph: Option<Arc<KnowledgeGraph>>,
    /// Mutable Plan-Do-Study-Act cycles managed through the JSON API.
    pub pdsa_cycles: Arc<RwLock<Vec<crate::routes::PdsaCycle>>>,
    /// The RAG generation engine (available when ML feature is enabled).
    #[cfg(feature = "ml")]
    pub rag_engine: Option<Arc<RagEngine>>,
}

impl AppState {
    /// Construct a new state from an arc'd archive.
    pub fn new(archive: Arc<Archive>) -> Self {
        Self {
            archive,
            static_dir: None,
            knowledge_graph: None,
            pdsa_cycles: Arc::new(RwLock::new(Vec::new())),
            #[cfg(feature = "ml")]
            rag_engine: None,
        }
    }

    /// Builder: enable static-file serving from `dir`.
    pub fn with_static_dir(mut self, dir: PathBuf) -> Self {
        self.static_dir = Some(dir);
        self
    }

    /// Builder: attach the knowledge graph.
    pub fn with_knowledge_graph(mut self, kg: Arc<KnowledgeGraph>) -> Self {
        self.knowledge_graph = Some(kg);
        self
    }

    /// Builder: attach the ML RAG engine.
    #[cfg(feature = "ml")]
    pub fn with_rag_engine(mut self, engine: Arc<RagEngine>) -> Self {
        self.rag_engine = Some(engine);
        self
    }
}
