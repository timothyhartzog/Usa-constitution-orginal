# Rust + Dioxus Pipeline Build Plan

This migration keeps source text and canonical JSON outside SurrealDB. The
database is a derived query layer that can be rebuilt from disk.

Last updated: 2026-05-12 15:58 EDT.

## Goal

Build an enterprise-ready Rust pipeline and Dioxus desktop/WASM research
workbench for the constitutional archive while preserving all downloaded text
as canonical files on disk.

The system should eventually support:

- Rust-native ingestion, cleaning, chunking, indexing, export, and validation.
- Local Ollama embeddings with `bge-m3`.
- Local parsing/enrichment with `qwen3.6:35b` or the best available Qwen model.
- SurrealDB as a rebuildable derived layer for vectors, graph traversal, and
  query acceleration.
- Dioxus desktop and WASM UI over the Rust archive, vector index, graph, and
  pipeline status.

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

## Completed Foundation

- Added `crates/constitution-pipeline`.
- Added canonical path contracts for raw, clean, chunk, graph, vector, index,
  and Founders Online data.
- Added Rust snapshot support for canonical corpus files.
- Added Rust corpus validation for `data/chunks/constitution_full_corpus.json`.
- Added `crates/constitution-app`.
- Added Dioxus shell targeting desktop and WASM.
- Verified:
  - `cargo check -p constitution-pipeline`
  - `cargo run -q -p constitution-pipeline -- validate`
  - `cargo check -p constitution-app --features desktop`
  - `cargo check -p constitution-app --features web --target wasm32-unknown-unknown`

Validation observed:

```text
chunks: 41,993
documents: 10,971
collections: 9
authors: 216
empty_text_chunks: 0
duplicate_chunk_ids: 0
warnings: none
```

## Remaining Work

### 1. Deterministic Pipeline Ports

Port these first because they are low-risk and easy to verify:

- `scripts/build_search_index.py`
- `scripts/export_csv.py`
- `scripts/clean_text.py`
- `scripts/chunk_documents.py`

Each Rust implementation should first write parallel outputs, then compare
counts, ids, schemas, and selected content against the existing Python output.

### 2. Fetch And Import Ports

Port network/live-data stages only after deterministic stages are stable:

- `scripts/ingest_sources.py`
- `scripts/fetch_world_constitutions.py`
- `scripts/fetch_eu_constitutions.py`
- `scripts/fetch_letters_delegates_loc.py`
- `scripts/fetch_founders_online.py`
- `scripts/founders_browser_sink.py`
- `scripts/import_founders_browser_batch.py`

These stages must be resumable and must never delete already downloaded files.

### 3. Vector Pipeline

Add a Rust/Ollama stage:

```text
read data/chunks/constitution_full_corpus.json
call Ollama /api/embed with bge-m3
write data/research_vectors/chunk_vectors.jsonl
record chunk_id, model, dims, text checksum, embedding
resume safely
```

### 4. SurrealDB Derived Rebuild

Add a disposable database rebuild command:

```bash
cargo run -p constitution-pipeline -- rebuild-surreal
```

It should:

- read canonical JSON/JSONL/documents from disk
- recreate schema and indexes
- import documents, chunks, vectors, nodes, and edges
- verify counts against source files
- avoid becoming the only copy of any data

### 5. Graph Generation

Canonical graph data should remain in JSONL and be imported into SurrealDB.

Important relations:

- `chunk -> belongs_to -> document`
- `chunk -> mentions -> person`
- `chunk -> cites -> clause`
- `chunk -> tagged_as -> issue`
- `chunk -> supports -> event`
- `document -> authored_by -> person`
- `document -> part_of -> source_collection`

### 6. Dioxus Workbench

Connect the shell to real data:

- search view
- document reader
- chunk detail view
- filters
- citation graph view
- vector similarity view
- pipeline status and validation view
- SurrealDB health/rebuild panel

### 7. Enterprise Hardening

- schema versioning
- migration reports
- structured rebuild logs
- checkpoint/resume files
- checksum verification
- benchmark suite
- CI for Rust pipeline, server, desktop, and WASM
- tracing and clear failure recovery

## Guardrail

No Rust stage should delete or overwrite raw downloads. New implementations
write parallel outputs until parity tests prove they match the current pipeline.

SurrealDB is always rebuildable. Canonical storage remains JSON, JSONL, plain
text, and original downloaded documents on disk.
