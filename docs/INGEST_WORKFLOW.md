# Ingest workflow (`.github/workflows/ingest.yml`)

GitHub-hosted Actions runners have open egress to founders.archives.gov,
gutenberg.org, archive.org, oll.libertyfund.org, avalon.law.yale.edu,
and the other public-domain hosts that the local sandbox blocks.
This workflow runs the full corpus pipeline on a runner and commits
the results back to the same branch.

## Trigger

From the GitHub UI: **Actions → "ingest" → "Run workflow"**.

From the CLI:

```bash
gh workflow run ingest.yml \
    --ref claude/constitution-archive-wasm-F1cRt \
    -f force=false \
    -f commit_back=true
```

| Input | Default | Effect |
|---|---|---|
| `force` | `false` | Pass `--force` to `ingest_sources.py` so cached payloads are re-downloaded. |
| `commit_back` | `true` | Stage and push the resulting `data/raw/`, `data/clean/`, `data/chunks/`, `data/index/search_index.json`, and `data/{acquisition,clean}_report.json` to the source branch. Set to `false` for a dry-run that just produces the step-summary report. |

## What the runner does

1. **Checkout** the triggering branch with full credentials (so the
   workflow can push back).
2. **Set up Python 3.11** + install `requirements.txt` (requests,
   beautifulsoup4, pytest).
3. **Set up Rust stable** with the `wasm32-unknown-unknown` target +
   `wasm-bindgen-cli@0.2.121`.
4. **Pre-flight URL probe** — `HEAD` every `source_url` in the
   manifest and print a small report. Doesn't fail the build.
5. **`scripts/ingest_sources.py`** — fetches every manifest entry
   into `data/raw/`. Failures are recorded in
   `data/acquisition_report.json` and surfaced as a Markdown table on
   the run's step summary. Tolerates partial failure.
6. **`scripts/clean_text.py`** — strips boilerplate / HTML, writes
   `data/clean/`. Skips entries missing raw text.
7. **`scripts/chunk_documents.py`** — applies the per-document chunk
   strategy and writes `data/chunks/constitution_full_corpus.json`.
   Skips entries missing cleaned text.
8. **`scripts/build_search_index.py`** — rebuilds the static JSON
   index used by the legacy frontend.
9. **`pytest -q`** — schema and content tests, including
   `test_manifest_schema.py`, `test_delegates.py`, `test_metadata.py`,
   `test_uniqueness.py`, `test_urls.py`, `test_content_quality.py`.
10. **`cargo run --release --bin constitution-archive -- build`** —
    rebuilds the binary archive consumed by the WASM frontend and
    `constitution-server`.
11. **wasm32 build + `wasm-bindgen`** — produces
    `frontend/wasm/pkg/` (gitignored — regenerated each run).
12. **Headless WASM smoke test** (`scripts/wasm_smoke_test.mjs`) —
    33 assertions against the freshly built bundle and archive.
13. **Coverage report** (`scripts/coverage_report.py`) — appended
    to the step summary so the run shows which delegates landed.
14. **Stage** the changed text/index files (the binary archive and
    the wasm pkg are gitignored on purpose — they're large and
    regenerated cheaply).
15. **Commit + push** if there are changes, with a message naming
    the workflow run id and the success/failure totals from the
    acquisition report.

## What is committed

- `data/raw/<collection>/<doc>/source.{html,txt}` and `metadata.json`
- `data/clean/<doc>.txt`
- `data/chunks/constitution_full_corpus.json`
- `data/index/search_index.json`
- `data/acquisition_report.json`, `data/clean_report.json`

What is **not** committed:

- `data/index/constitution_archive.bin` — large binary, regenerable
  with `cargo run --release --bin constitution-archive -- build`.
- `frontend/wasm/pkg/` — regenerable with `./scripts/build_wasm.sh`.
- `target/` — Rust build artifacts.

## Concurrency / safety

- `concurrency.group` keys on the branch ref so two ingest runs on
  the same branch cannot race each other.
- `permissions: contents: write` is the only privilege required.
- The workflow never force-pushes; it commits cleanly on top of HEAD
  and pushes with the default fast-forward semantics.
- `pytest` and `wasm_smoke_test` run **before** the commit step, so a
  malformed source that breaks the build aborts the workflow with no
  state change to the branch.

## Re-deriving locally after a successful run

```bash
git pull
cargo run --release --bin constitution-archive -- build
./scripts/build_wasm.sh        # rebuilds frontend/wasm/pkg + smoke-tests
python3 scripts/coverage_report.py
```
