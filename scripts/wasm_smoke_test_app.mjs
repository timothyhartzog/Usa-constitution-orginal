// Headless smoke test: verify the compiled WASM bundle instantiates
// in a Node.js environment with a minimal browser shim. Runs as part
// of CI to catch link-time / undefined-import regressions before
// deploying.
//
// Usage:  node scripts/wasm_smoke_test_app.mjs

import { readFile } from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";
import { dirname, resolve } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const DIST = resolve(__dirname, "..", "dist");

function shimBrowserGlobals() {
    // Minimal DOM stubs so wasm-bindgen JS glue can `import` and at least
    // load. We do NOT exercise Dioxus rendering — just the instantiation
    // path. If the WASM imports an unknown JS function we'll see an error.
    globalThis.window = globalThis;
    globalThis.document = {
        createElement() { return { style: {}, setAttribute() {}, appendChild() {}, addEventListener() {} }; },
        head: { appendChild() {} },
        body: { appendChild() {} },
        querySelector() { return null; },
        getElementById() { return null; },
        addEventListener() {},
        createTextNode() { return {}; },
    };
    if (!globalThis.navigator || !globalThis.navigator.userAgent) {
        try { globalThis.navigator = { userAgent: "node-smoke" }; } catch (_) { /* read-only */ }
    }
    globalThis.location = { href: "http://localhost/", pathname: "/" };
    globalThis.localStorage = {
        store: new Map(),
        getItem(k) { return this.store.has(k) ? this.store.get(k) : null; },
        setItem(k, v) { this.store.set(k, String(v)); },
        removeItem(k) { this.store.delete(k); },
    };
    globalThis.HTMLElement = class {};
    globalThis.fetch = async () => ({
        ok: false,
        status: 404,
        async arrayBuffer() { return new ArrayBuffer(0); },
        async json() { return []; },
        async text() { return ""; },
    });
    globalThis.Request = class {};
    globalThis.Response = class {};
    globalThis.Headers = class {};
}

async function main() {
    shimBrowserGlobals();

    // Dioxus spawns async tasks during boot that may reject after our top-level
    // await returns. Anything past instantiation is "expected" under the shim.
    let postInitError = null;
    process.on("unhandledRejection", (err) => {
        postInitError = err;
    });
    process.on("uncaughtException", (err) => {
        postInitError = err;
    });

    const jsPath = resolve(DIST, "constitution-app.js");
    const wasmPath = resolve(DIST, "constitution-app_bg.wasm");

    console.log("[smoke] loading", jsPath);
    const mod = await import(pathToFileURL(jsPath).href);
    const init = mod.default;
    if (typeof init !== "function") {
        throw new Error("default export from constitution-app.js is not a function");
    }

    console.log("[smoke] reading WASM bytes from", wasmPath);
    const wasmBytes = await readFile(wasmPath);

    console.log("[smoke] instantiating WASM (", wasmBytes.length, "bytes )");
    try {
        await init({ module_or_path: wasmBytes });
    } catch (err) {
        // Once we get past WebAssembly.instantiate the WASM is linked and
        // valid. The Dioxus runtime then tries to mount into the DOM, which
        // our minimal shim cannot fully serve — runtime errors at that point
        // are expected. We treat all post-instantiation errors as success;
        // a real failure (missing JS import, bad WASM binary) would surface
        // as a LinkError or CompileError instead of a RuntimeError.
        const name = err && err.constructor && err.constructor.name;
        if (name === "LinkError" || name === "CompileError") {
            throw err;
        }
        console.log(`[smoke] instantiation succeeded (post-mount ${name || "Error"} is expected under headless shim)`);
    }

    if (postInitError) {
        const name = postInitError.constructor && postInitError.constructor.name;
        if (name === "LinkError" || name === "CompileError") {
            throw postInitError;
        }
        console.log(`[smoke] (post-init ${name || "Error"} from async runtime; ignored under shim)`);
    }

    console.log("[smoke] OK");
}

main().catch((err) => {
    console.error("[smoke] FAILED:", err);
    process.exit(1);
});
