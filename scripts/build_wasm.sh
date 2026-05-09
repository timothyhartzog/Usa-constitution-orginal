#!/usr/bin/env bash
# Build the binary archive + the WebAssembly bundle and place them where
# the frontend expects them.
#
# Idempotent. Requires:
#   - a Rust toolchain with the wasm32-unknown-unknown target
#       (rustup target add wasm32-unknown-unknown)
#   - wasm-bindgen-cli matching the wasm-bindgen version in Cargo.toml
#       (cargo install --version 0.2.121 wasm-bindgen-cli)
#   - optionally Node.js for the headless smoke test

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." >/dev/null 2>&1 && pwd)"
cd "${REPO_ROOT}"

WASM_TARGET="wasm32-unknown-unknown"
WASM_OUT_DIR="${REPO_ROOT}/frontend/wasm/pkg"

echo "==> Building binary archive"
cargo run --release --bin constitution-archive -- build \
    --corpus data/chunks/constitution_full_corpus.json \
    --timeline data/process_timeline.json \
    --output data/index/constitution_archive.bin

if ! rustup target list --installed | grep -q "^${WASM_TARGET}$"; then
    echo "==> Adding rustup target ${WASM_TARGET}"
    rustup target add "${WASM_TARGET}"
fi

echo "==> Compiling constitution-wasm to ${WASM_TARGET}"
cargo build --release --target "${WASM_TARGET}" -p constitution-wasm

if ! command -v wasm-bindgen >/dev/null 2>&1; then
    echo "wasm-bindgen-cli not installed — install with:"
    echo "    cargo install --version 0.2.121 wasm-bindgen-cli"
    echo "Skipping JS-glue generation. The compiled .wasm is at"
    echo "    target/${WASM_TARGET}/release/constitution_wasm.wasm"
    exit 0
fi

echo "==> Generating JS bindings into ${WASM_OUT_DIR}"
mkdir -p "${WASM_OUT_DIR}"
wasm-bindgen \
    --target web \
    --out-dir "${WASM_OUT_DIR}" \
    --out-name constitution_wasm \
    "target/${WASM_TARGET}/release/constitution_wasm.wasm"

echo "==> Done."
echo "    Serve with: python3 -m http.server 8000"
echo "    Then open:  http://localhost:8000/frontend/wasm/"

if command -v node >/dev/null 2>&1; then
    echo "==> Running headless smoke test"
    node scripts/wasm_smoke_test.mjs
fi
