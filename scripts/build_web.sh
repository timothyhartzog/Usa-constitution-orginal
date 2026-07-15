#!/usr/bin/env bash
#
# Build the constitution-app WASM bundle for production / static hosting.
#
# Output: dist/
#   index.html                  - boot loader, registers service worker
#   manifest.webmanifest        - PWA manifest
#   icon.svg                    - PWA / favicon
#   service-worker.js           - caches shell + archive for offline use
#   constitution-app.js         - wasm-bindgen JS glue
#   constitution-app_bg.wasm    - compiled Rust -> WASM
#   snippets/                   - JS shims emitted by wasm-bindgen
#   assets/main.css             - app stylesheet
#   assets/world_meta.json      - world constitution metadata
#   assets/constitution_archive.bin
#                               - binary search archive
#   assets/constitution_archive.bin.gz
#                               - gzip-compressed archive for pre-compressed
#                                 serving (e.g. `gzip_static on;` on nginx,
#                                 or matching response handler on a CDN)
#
# Prerequisites:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version 0.2.121 --locked
#
# Optional:
#   apt install binaryen        # for `wasm-opt` (set WASM_OPT=true)
#
# Usage:
#   scripts/build_web.sh                          # full build
#   scripts/build_web.sh --skip-archive           # reuse existing archive
#   WASM_OPT=true scripts/build_web.sh            # additional wasm-opt pass
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

DIST="$ROOT/dist"
ASSETS_DIR="$DIST/assets"

SKIP_ARCHIVE=0
for arg in "$@"; do
    case "$arg" in
        --skip-archive) SKIP_ARCHIVE=1 ;;
        --release) ;;  # no-op, build is always release
        *) echo "unknown argument: $arg" >&2; exit 1 ;;
    esac
done

echo "==> Cleaning dist/"
rm -rf "$DIST"
mkdir -p "$ASSETS_DIR"

echo "==> Cargo build (wasm32-unknown-unknown, release)"
cargo build \
    -p constitution-app \
    --features web \
    --target wasm32-unknown-unknown \
    --release

WASM_IN="$ROOT/target/wasm32-unknown-unknown/release/constitution-app.wasm"
echo "    raw wasm:  $(du -h "$WASM_IN" | cut -f1)"

echo "==> wasm-bindgen --target web"
wasm-bindgen \
    --target web \
    --out-dir "$DIST" \
    --out-name constitution-app \
    --no-typescript \
    "$WASM_IN"

WASM_OUT="$DIST/constitution-app_bg.wasm"
echo "    bound wasm: $(du -h "$WASM_OUT" | cut -f1)"

if [ "${WASM_OPT:-false}" = "true" ]; then
    if command -v wasm-opt >/dev/null 2>&1; then
        echo "==> wasm-opt -Oz"
        if wasm-opt -Oz "$WASM_OUT" -o "$WASM_OUT.opt" 2>/dev/null; then
            mv "$WASM_OUT.opt" "$WASM_OUT"
            echo "    optimized:  $(du -h "$WASM_OUT" | cut -f1)"
        else
            rm -f "$WASM_OUT.opt"
            echo "    wasm-opt rejected the binary (likely too-old binaryen); skipping"
        fi
    else
        echo "    wasm-opt not found; skipping (install binaryen)"
    fi
fi

if [ $SKIP_ARCHIVE -eq 0 ]; then
    if [ ! -f "$ROOT/data/index/constitution_archive.bin" ] || [ ! -f "$ROOT/crates/constitution-app/assets/world_meta.json" ]; then
        echo "==> Building constitution archive (web profile)"
        cargo run -p constitution-cli --bin build-archive --release -- --web --window-size 500 --stride 450
    else
        echo "==> Archive already built; reusing"
    fi
else
    echo "==> Skipping archive build (--skip-archive)"
fi

echo "==> Copying static assets"
cp "$ROOT/crates/constitution-app/assets/main.css" "$ASSETS_DIR/"
if [ -f "$ROOT/crates/constitution-app/assets/world_meta.json" ]; then
    cp "$ROOT/crates/constitution-app/assets/world_meta.json" "$ASSETS_DIR/"
fi
if [ -f "$ROOT/data/index/knowledge_graph.json" ]; then
    cp "$ROOT/data/index/knowledge_graph.json" "$ASSETS_DIR/"
fi
if [ -f "$ROOT/data/index/constitution_archive.bin" ]; then
    cp "$ROOT/data/index/constitution_archive.bin" "$ASSETS_DIR/"
    echo "==> Pre-compressing archive (gzip -9)"
    gzip -9 -k -f "$ASSETS_DIR/constitution_archive.bin"
    echo "    archive:    $(du -h "$ASSETS_DIR/constitution_archive.bin" | cut -f1)"
    echo "    archive.gz: $(du -h "$ASSETS_DIR/constitution_archive.bin.gz" | cut -f1)"
fi

cp "$ROOT/crates/constitution-app/assets/service-worker.js" "$DIST/"
cp "$ROOT/crates/constitution-app/assets/manifest.webmanifest" "$DIST/"
cp "$ROOT/crates/constitution-app/assets/icon.svg" "$DIST/"

echo "==> Writing index.html"
cat > "$DIST/index.html" <<'HTML'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Constitution Research Workbench</title>
    <meta name="description" content="WebAssembly-powered research platform for the U.S. Constitution, the Federalist Papers, and 194 national constitutions.">
    <meta name="theme-color" content="#28483a">
    <link rel="manifest" href="manifest.webmanifest">
    <link rel="icon" type="image/svg+xml" href="icon.svg">
    <link rel="apple-touch-icon" href="icon.svg">
    <link rel="stylesheet" href="assets/main.css">
    <link rel="preload" href="constitution-app_bg.wasm" as="fetch" type="application/wasm" crossorigin>
    <link rel="preload" href="assets/constitution_archive.bin" as="fetch" type="application/octet-stream" crossorigin>
    <style>
        body { margin: 0; font-family: Inter, ui-sans-serif, system-ui, -apple-system, sans-serif; }
        #boot-loader {
            position: fixed; inset: 0;
            display: flex; flex-direction: column;
            align-items: center; justify-content: center;
            background: #f7f5ef; color: #17211c;
            z-index: 9999;
        }
        #boot-loader.hidden { display: none; }
        .boot-spinner {
            width: 48px; height: 48px;
            border: 4px solid #d8d2c3;
            border-top-color: #28483a;
            border-radius: 50%;
            animation: boot-spin 0.8s linear infinite;
            margin-bottom: 18px;
        }
        @keyframes boot-spin { to { transform: rotate(360deg); } }
        .boot-title { font-size: 18px; font-weight: 600; margin: 0 0 4px; }
        .boot-subtitle { font-size: 13px; color: #5f665f; margin: 0; }
        #boot-error {
            display: none;
            margin-top: 16px; padding: 12px 16px;
            border: 1px solid #c4452f; background: #fef2f2;
            color: #991b1b; max-width: 480px; border-radius: 6px;
            font-size: 13px; line-height: 1.5;
        }
        @media (prefers-color-scheme: dark) {
            #boot-loader { background: #0f1729; color: #e8ecf2; }
            .boot-subtitle { color: #9aa3b2; }
        }
    </style>
</head>
<body>
    <div id="boot-loader">
        <div class="boot-spinner"></div>
        <p class="boot-title">Constitution Research Workbench</p>
        <p class="boot-subtitle">Initializing WebAssembly runtime...</p>
        <div id="boot-error"></div>
    </div>

    <script type="module">
        import init from "./constitution-app.js";
        async function boot() {
            try {
                await init();
                const loader = document.getElementById("boot-loader");
                if (loader) { loader.classList.add("hidden"); }
            } catch (err) {
                console.error("Failed to start app:", err);
                const errEl = document.getElementById("boot-error");
                if (errEl) {
                    errEl.style.display = "block";
                    errEl.innerHTML = "<strong>Failed to start app.</strong><br>" + String(err);
                }
            }
        }
        boot();

        if ("serviceWorker" in navigator) {
            window.addEventListener("load", () => {
                navigator.serviceWorker.register("./service-worker.js").catch((err) => {
                    console.warn("Service worker registration failed:", err);
                });
            });
        }
    </script>
</body>
</html>
HTML

echo
echo "==> Build complete."
echo "    dist/                                  $(du -sh "$DIST" | cut -f1)"
echo "    dist/constitution-app_bg.wasm          $(du -h "$WASM_OUT" | cut -f1)"
if [ -f "$ASSETS_DIR/constitution_archive.bin.gz" ]; then
    echo "    dist/assets/constitution_archive.bin    $(du -h "$ASSETS_DIR/constitution_archive.bin" | cut -f1)"
    echo "    dist/assets/constitution_archive.bin.gz $(du -h "$ASSETS_DIR/constitution_archive.bin.gz" | cut -f1)"
fi
echo
echo "Serve locally:"
echo "    cd dist && python3 -m http.server 8000"
echo "    open  http://localhost:8000/"
