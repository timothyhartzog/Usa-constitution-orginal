# Constitutional Research System

This repository builds a local, static, full-text constitutional research system from public-domain primary sources. The pipeline downloads or reuses cached archival texts, normalizes them into a shared schema, chunks them into searchable passages, builds a client-side search index, and serves a browser interface with filtering and export tools.

## Included collections

- **U.S. Constitution** (Article I–VII) and the **Bill of Rights** (1791)
- **Madison's Notes of the Constitutional Convention** (key debate days)
- **Farrand's Records of the Federal Convention of 1787** (vols 1 + 2)
- **The Federalist Papers** (Hamilton, Madison, Jay; complete)
- **Anti-Federalist series** — Brutus, Letters from a Federal Farmer, Cato,
  Centinel, plus Mason's *Objections* and Henry's Virginia-convention speeches
- **State ratification documents** — Virginia, New York, North Carolina
- **Founders correspondence (1786–1791)** — Jefferson, Madison, Hamilton,
  Washington, John Adams, Jay, Franklin

The manifest at `config/sources_manifest.json` lists 25 documents over six
collections. The first eleven ship with cached `data/raw/` payloads; the
remainder are populated by `scripts/ingest_sources.py` when run with network
access (the pipeline gracefully skips manifest entries whose raw source has
not yet been fetched).

## Data layout

The build produces these repository artifacts:

- `data/raw/` - cached source downloads grouped by collection and document
- `data/clean/` - normalized plain-text working files
- `data/chunks/constitution_full_corpus.json` - chunked corpus with metadata
- `data/index/search_index.json` - client-side inverted index and filter metadata

Each chunk includes:

- `chunk_id`
- `document_id`
- `title`
- `author`
- `date`
- `source_collection`
- `source_url`
- `document_type`
- `issue_tags`
- `constitutional_clause_tags`
- `text`
- `word_count`

## Local setup

```bash
git clone <repository-url>
cd Usa-constitution-orginal

python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

## Rebuild the corpus

```bash
python3 scripts/ingest_sources.py
python3 scripts/clean_text.py
python3 scripts/chunk_documents.py
python3 scripts/build_search_index.py
python3 scripts/export_csv.py
pytest -q
```

## Run the browser interface

Because the frontend loads JSON with `fetch`, serve the repository through a simple local static server instead of opening `frontend/index.html` directly from the file system.

```bash
python3 -m http.server 8000
```

Then open `http://localhost:8000/frontend/`.

## Browser features

- full-text search
- filters for collection, document, author, and issue
- passage viewer with source link
- export selected chunks as JSON or CSV

## Rust + WebAssembly archive (preview)

A pure-Rust core (`crates/constitution-archive`) provides a binary,
versioned archive format with BM25 search, metadata filtering, and a
`ProcessTimeline` describing the drafting and ratification process. The
same crate compiles to native (consumed by `constitution-cli`) and to
`wasm32` (consumed by `constitution-wasm`) with no I/O assumptions in
the public API.

```text
crates/
├── constitution-archive   # core: chunks + inverted index + timeline + citations
├── constitution-wasm      # wasm-bindgen JS surface (WasmArchive class)
├── constitution-cli       # native CLI (build / search / process / stats / citations)
└── constitution-server    # Axum REST server over the binary archive
```

### Build the binary archive

```bash
cargo run --release --bin constitution-archive -- build
# → data/index/constitution_archive.bin (≈ 11 MB for the current corpus)
```

### Native CLI

```bash
cargo run --release --bin constitution-archive -- stats
cargo run --release --bin constitution-archive -- search "great compromise representation"
# Tolerate typos within edit distance 2 (BK-tree expansion):
cargo run --release --bin constitution-archive -- search --fuzzy 2 "great comprmise"
# Type-ahead suggestions over the indexed vocabulary:
cargo run --release --bin constitution-archive -- suggest ratif
# Find indexed terms within a Levenshtein distance of a query term:
cargo run --release --bin constitution-archive -- fuzzy --max-distance 2 federlism
cargo run --release --bin constitution-archive -- process phase ratification
cargo run --release --bin constitution-archive -- process get convention_great_compromise
# Citation graph:
cargo run --release --bin constitution-archive -- citations top --limit 25
cargo run --release --bin constitution-archive -- citations from us_constitution_1787_article_1_0000
cargo run --release --bin constitution-archive -- citations to clause:I.8
cargo run --release --bin constitution-archive -- citations to person:madison
```

### Axum REST server

`constitution-server` loads the binary archive once at startup and exposes
a read-only REST surface plus optional static-file serving for the
frontend. Single-process, no database, designed for stateless container
deployment.

```bash
cargo run --release -p constitution-server -- \
    --addr 127.0.0.1:8080 \
    --static-dir frontend
# → 127.0.0.1:8080/         frontend (with the WASM page at /wasm/)
# → 127.0.0.1:8080/healthz  liveness probe
# → 127.0.0.1:8080/api/...  REST surface (see below)
```

Endpoints (all return JSON; `?` queries unless noted):

| Endpoint | Purpose |
|---|---|
| `GET  /healthz`                          | liveness + chunk count |
| `GET  /api/stats`                        | full archive stats |
| `POST /api/search`                       | BM25 + fuzzy + filters + snippets |
| `GET  /api/suggest?prefix=…`             | type-ahead over vocabulary |
| `GET  /api/fuzzy?term=…&max_distance=2`  | indexed terms within Levenshtein distance |
| `GET  /api/chunk/:id`                    | single chunk |
| `GET  /api/process`                      | full process timeline |
| `GET  /api/process/:id`                  | single timeline event |
| `GET  /api/process/phase/:name`          | events in a phase |
| `GET  /api/process/search?q=…`           | timeline free-text search |
| `GET  /api/citations/top?limit=N`        | top-N most-cited targets |
| `GET  /api/citations/from/:chunk_id`     | outgoing citations of a chunk |
| `GET  /api/citations/to/:target_key`     | incoming citations to a target |

### Container

```bash
docker build -t constitution-server .
docker run --rm -p 8080:8080 constitution-server
# → http://localhost:8080/wasm/
```

The Dockerfile builds the binary archive, the wasm32 frontend bundle, and
the server binary in one builder stage and ships the result on
`distroless/cc-debian12:nonroot`.

### WebAssembly bundle (browser, fully offline)

The build is verified end-to-end by `scripts/wasm_smoke_test.mjs`, which
loads the bundle in Node and runs sixteen assertions through the live
WASM `WasmArchive` class:

```bash
# One-shot prerequisites
rustup target add wasm32-unknown-unknown
cargo install --version 0.2.121 wasm-bindgen-cli

# Build + smoke-test
./scripts/build_wasm.sh
# → data/index/constitution_archive.bin (≈ 11 MB)
# → frontend/wasm/pkg/constitution_wasm_bg.wasm (≈ 380 KB)
# → 16 smoke-test assertions through the live wasm32 module

# Serve the page
python3 -m http.server 8000
# open http://localhost:8000/frontend/wasm/
```

The CI workflow at `.github/workflows/wasm-build.yml` runs the same
build + smoke test on every PR and rejects bundles larger than 1 MB.

Until the bundle has been built the WASM page falls back to the JSON
timeline view, so the page is always usable.

### Process explorer

The `data/process_timeline.json` file enumerates the constitutional
process — Annapolis Convention through ratification of the Bill of
Rights — with cross-references to the underlying chunk ids in the
archive, so the UI can pivot from "Massachusetts Compromise" → the
ratification-debate passages it cites.

## Public-domain provenance

See `docs/SOURCES.md` for collection notes, source URLs, and build caveats.

## License

Repository code is released under the terms in `LICENSE`. The source texts referenced by this project are public-domain materials from archival and historical repositories.
