//! Searchable archive of U.S. Constitution-era primary sources.
//!
//! This crate is the pure-Rust core of the Constitutional Research System.
//! It is `wasm32`-compatible (no I/O, no threads, no `std::time`-dependent
//! types in public API) and is consumed by both the native CLI
//! (`constitution-cli`) and the browser bundle (`constitution-wasm`).
//!
//! # Pipeline
//!
//! ```text
//!   data/chunks/constitution_full_corpus.json
//!     │  (built once, offline)
//!     ▼
//!   Archive::from_chunks()  →  bincode bytes  →  Archive::load()
//!     │                                            │
//!     │ build inverted index (BM25)                │
//!     │ build metadata facets                      │
//!     │ load process timeline                      │
//!     ▼                                            ▼
//!   Archive::search(...)                       Archive::process_lookup(...)
//! ```
//!
//! The archive bytes are deterministic and can be fetched once by the
//! browser, cached in `IndexedDB`, and queried entirely client-side.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![warn(missing_docs)]

pub mod archive;
pub mod chunk;
pub mod error;
pub mod filter;
pub mod fuzzy;
pub mod index;
pub mod process;
pub mod snippet;
pub mod tokenizer;

pub use archive::{Archive, ArchiveStats};
pub use chunk::{Chunk, ChunkId};
pub use error::ArchiveError;
pub use filter::{Filter, FilterValue};
pub use fuzzy::{levenshtein, BkTree};
pub use index::{SearchHit, SearchOptions};
pub use process::{ProcessEvent, ProcessPhase, ProcessTimeline};
pub use snippet::{make_snippet, Snippet};
