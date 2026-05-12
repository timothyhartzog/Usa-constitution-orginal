# Session State

Saved: 2026-05-12 15:58 EDT.

Workspace:

```text
/Users/timothyhartzog/code/rust-local/Usa-constitution-orginal
```

## Current Direction

Build the project toward an enterprise-ready Rust pipeline and Dioxus
desktop/WASM application.

Core architectural decisions:

- Keep canonical data outside SurrealDB.
- Treat SurrealDB as a rebuildable derived index/cache for vector search,
  graph traversal, and query acceleration.
- Preserve all downloaded text and JSON/JSONL artifacts on disk.
- Use Rust for the pipeline and runtime.
- Use Dioxus for the desktop/WASM app.
- Use Ollama locally:
  - `bge-m3` for embeddings/vectors.
  - `qwen3.6:35b` or best available Qwen model for parsing/enrichment.

## Files Added Or Updated In This Session

- `Cargo.toml`
- `Cargo.lock`
- `crates/constitution-pipeline/`
- `crates/constitution-app/`
- `docs/RUST_DIOXUS_PIPELINE_PLAN.md`
- `docs/SESSION_STATE.md`
- `data/pipeline/snapshot.json`

Note: many raw data files may also be changing because downloads/importers were
active or recently active. Do not delete or revert them.

## What Was Built

`constitution-pipeline`:

- canonical path model
- file snapshot command
- snapshot verification command
- corpus validation command

`constitution-app`:

- Dioxus shell
- desktop feature target
- web/WASM feature target
- initial workbench layout for search, documents, graph, vectors, and pipeline

## Verified Commands

These passed during the session:

```bash
cargo check -p constitution-pipeline
cargo run -q -p constitution-pipeline -- validate
cargo check -p constitution-app --features desktop
cargo check -p constitution-app --features web --target wasm32-unknown-unknown
```

Validation output observed:

```text
chunks: 41,993
documents: 10,971
collections: 9
authors: 216
empty_text_chunks: 0
duplicate_chunk_ids: 0
warnings: none
```

## Known Caveats

- Snapshot verification may fail while raw downloads are still changing. That is
  expected and means the drift detector is working.
- A `git status --short` attempt failed with a Git LFS clean-filter permission
  error on `data/chunks/constitution_full_corpus.json`. Avoid assuming git
  status is reliable until that local LFS issue is handled.
- There were pre-existing modifications in other Rust files before/around the
  Dioxus and pipeline work. Do not revert unrelated user changes.

## Next Best Step

Port the first deterministic Python stage into Rust:

```text
scripts/build_search_index.py -> crates/constitution-pipeline
```

Do it safely:

1. Read existing Python script and output schema.
2. Implement Rust equivalent.
3. Write output to a parallel path first, such as `data/rust_index/`.
4. Compare term counts, filters, postings, issue index, clause index, and
   document metadata against `data/index/search_index.json`.
5. Only after parity is proven, decide whether to replace the Python stage.

After that, port:

```text
scripts/export_csv.py
scripts/clean_text.py
scripts/chunk_documents.py
```

## Resume Prompt

Use this prompt after reopening:

```text
Read docs/SESSION_STATE.md and docs/RUST_DIOXUS_PIPELINE_PLAN.md, then continue
the Rust migration by porting scripts/build_search_index.py into
crates/constitution-pipeline with parallel output and parity checks. Do not
delete or overwrite raw downloaded text.
```
