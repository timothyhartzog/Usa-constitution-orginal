#!/usr/bin/env bash
# Build the binary archive + the WebAssembly bundle and place them where
# the frontend expects them.
#
# Idempotent. Requires:
#   - a Rust toolchain (cargo)
#   - wasm-pack (install: `cargo install wasm-pack`)

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." >/dev/null 2>&1 && pwd)"
cd "${REPO_ROOT}"

echo "==> Building binary archive"
cargo run --release --bin constitution-archive -- build \
    --corpus data/chunks/constitution_full_corpus.json \
    --timeline data/process_timeline.json \
    --output data/index/constitution_archive.bin

if ! command -v wasm-pack >/dev/null 2>&1; then
    echo "wasm-pack not installed — install with: cargo install wasm-pack"
    echo "Skipping WASM bundle build."
    exit 0
fi

echo "==> Building WebAssembly bundle"
wasm-pack build crates/constitution-wasm \
    --release \
    --target web \
    --out-dir "${REPO_ROOT}/frontend/wasm/pkg" \
    --out-name constitution_wasm

echo "==> Done. Serve with: python3 -m http.server 8000"
echo "    Then open: http://localhost:8000/frontend/wasm/"
