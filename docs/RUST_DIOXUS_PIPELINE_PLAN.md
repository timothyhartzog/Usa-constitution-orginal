# Rust + Dioxus Pipeline Plan

This migration keeps source text and canonical JSON outside SurrealDB. The
database is a derived query layer that can be rebuilt from disk.

## Storage Contract

- `data/raw/` stores immutable source downloads.
- `data/clean/` stores normalized plain text.
- `data/chunks/` stores canonical chunk JSON.
- `data/research_graph/` stores canonical graph JSONL.
- `data/research_vectors/` stores canonical embedding JSONL.
- `data/index/` stores rebuildable search/archive artifacts.
- SurrealDB stores imported copies for vector search, graph traversal, and
  full-text querying.

## Rust Crates

- `constitution-archive`: pure Rust archive/search engine.
- `constitution-pipeline`: canonical paths, snapshots, validation, and future
  ingestion stages.
- `constitution-app`: Dioxus shell for desktop and WASM.
- `constitution-server`: Axum API over the archive.
- `constitution-wasm`: low-level wasm-bindgen archive bindings.

## First Milestones

1. Snapshot canonical data files with `constitution-pipeline snapshot`.
2. Validate `constitution_full_corpus.json` from Rust.
3. Port deterministic Python stages one at a time into `constitution-pipeline`.
4. Add a SurrealDB rebuild command that imports JSON/JSONL and vectors.
5. Connect the Dioxus shell to local archive search, then vectors and graph.

## Guardrail

No Rust stage should delete or overwrite raw downloads. New implementations
write parallel outputs until parity tests prove they match the current pipeline.
