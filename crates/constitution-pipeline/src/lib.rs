//! Rust-first pipeline contracts for the constitutional research corpus.
//!
//! The pipeline treats downloaded text and generated JSON/JSONL artifacts as
//! canonical files on disk. Databases, vector indexes, and UI caches are
//! derived from these files and must be rebuildable.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

pub mod paths;
pub mod snapshot;
pub mod validate;

pub use paths::PipelinePaths;
pub use snapshot::{Snapshot, SnapshotRecord};
pub use validate::{CorpusSummary, ValidationReport};
