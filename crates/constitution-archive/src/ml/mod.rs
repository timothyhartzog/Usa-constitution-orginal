pub mod rag;
pub mod rerank;
pub mod vector;

pub use rag::{RagConfig, RagEngine};
pub use rerank::{RerankResult, Reranker};
pub use vector::{VectorIndex, VectorResult};

/// Common ML Error types
#[derive(thiserror::Error, Debug)]
pub enum MlError {
    #[error("Candle execution error: {0}")]
    Candle(#[from] candle_core::Error),
    #[error("Tokenizer error: {0}")]
    Tokenizer(String),
    #[error("Vector index error: {0}")]
    Vector(String),
    #[error("Model load error: {0}")]
    ModelLoad(String),
}
