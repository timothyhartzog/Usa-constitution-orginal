// Service worker for the Constitution Research Workbench.
//
// Caching strategy:
//   1. App shell (HTML, JS glue, WASM bundle, CSS, world metadata, manifest):
//      cache-first with background revalidation. These are versioned via
//      the CACHE_VERSION below; on activation we drop old caches.
//   2. Binary archive (assets/constitution_archive.bin): network-first with
//      cache fallback. The archive is big — once a user has fetched it
//      successfully, we serve from cache to make subsequent loads instant.
//   3. Everything else: passthrough to the network.

const CACHE_VERSION = "v1";
const APP_CACHE = `constitution-app-shell-${CACHE_VERSION}`;
const DATA_CACHE = `constitution-app-data-${CACHE_VERSION}`;

const APP_SHELL = [
    "./",
    "./index.html",
    "./constitution-app.js",
    "./constitution-app_bg.wasm",
    "./assets/main.css",
    "./assets/world_meta.json",
    "./manifest.webmanifest",
];

self.addEventListener("install", (event) => {
    event.waitUntil(
        caches.open(APP_CACHE).then((cache) =>
            cache.addAll(APP_SHELL).catch((err) => {
                // Pre-cache failures are non-fatal; runtime fetches will retry.
                console.warn("[sw] pre-cache partial failure:", err);
            }),
        ),
    );
    self.skipWaiting();
});

self.addEventListener("activate", (event) => {
    event.waitUntil(
        caches
            .keys()
            .then((keys) =>
                Promise.all(
                    keys
                        .filter((k) => k !== APP_CACHE && k !== DATA_CACHE)
                        .map((k) => caches.delete(k)),
                ),
            )
            .then(() => self.clients.claim()),
    );
});

function isAppShell(url) {
    const path = url.pathname.replace(/^.*?(\/[^/]+)?\//, "/");
    return (
        path === "/" ||
        path.endsWith("/index.html") ||
        path.endsWith("/constitution-app.js") ||
        path.endsWith("/constitution-app_bg.wasm") ||
        path.endsWith("/main.css") ||
        path.endsWith("/world_meta.json") ||
        path.endsWith("/manifest.webmanifest")
    );
}

function isArchive(url) {
    return url.pathname.endsWith("/constitution_archive.bin");
}

self.addEventListener("fetch", (event) => {
    const req = event.request;
    if (req.method !== "GET") return;

    const url = new URL(req.url);

    if (isAppShell(url)) {
        event.respondWith(staleWhileRevalidate(req, APP_CACHE));
        return;
    }

    if (isArchive(url)) {
        event.respondWith(networkFirst(req, DATA_CACHE));
        return;
    }

    // Default: just try the network.
});

async function staleWhileRevalidate(req, cacheName) {
    const cache = await caches.open(cacheName);
    const cached = await cache.match(req);
    const network = fetch(req)
        .then((resp) => {
            if (resp && resp.ok) {
                cache.put(req, resp.clone()).catch(() => {});
            }
            return resp;
        })
        .catch(() => null);
    return cached || (await network) || new Response("offline", { status: 503 });
}

async function networkFirst(req, cacheName) {
    const cache = await caches.open(cacheName);
    try {
        const resp = await fetch(req);
        if (resp && resp.ok) {
            cache.put(req, resp.clone()).catch(() => {});
        }
        return resp;
    } catch (err) {
        const cached = await cache.match(req);
        if (cached) return cached;
        return new Response(`offline: ${err && err.message}`, { status: 503 });
    }
}
