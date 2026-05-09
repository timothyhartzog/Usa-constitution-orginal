#!/usr/bin/env node
// Headless smoke test for the constitution-wasm bundle.
//
// Loads frontend/wasm/pkg/constitution_wasm.js, then loads the binary
// archive at data/index/constitution_archive.bin, and runs a handful of
// representative queries. Exits non-zero if any assertion fails so this
// script is safe to wire into CI.

import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = resolve(HERE, "..");
const PKG = resolve(REPO, "frontend/wasm/pkg/constitution_wasm.js");
const ARCHIVE = resolve(REPO, "data/index/constitution_archive.bin");
const WASM = resolve(REPO, "frontend/wasm/pkg/constitution_wasm_bg.wasm");

function assert(cond, msg) {
  if (!cond) {
    console.error(`FAIL: ${msg}`);
    process.exit(1);
  }
  console.log(`ok   ${msg}`);
}

const mod = await import(PKG);
// `target=web` expects a fetch-able URL or bytes. Pass bytes directly.
await mod.default({ module_or_path: await readFile(WASM) });

const bytes = await readFile(ARCHIVE);
const archive = new mod.WasmArchive(new Uint8Array(bytes));

const stats = archive.stats();
assert(stats.chunks > 1000, `archive has ${stats.chunks} chunks (>1000)`);
assert(stats.events >= 35, `archive has ${stats.events} timeline events (>=35)`);
assert(stats.terms > 10000, `archive vocabulary has ${stats.terms} terms (>10000)`);

// 1. Plain BM25 search returns hits.
const plain = archive.search(JSON.stringify({
  query: "great compromise representation",
  limit: 5,
  fuzzy_distance: 0,
}));
assert(plain.length > 0, "plain search returns hits");
assert(plain[0].score > 0, "top hit has positive BM25 score");

// 2. Fuzzy search recovers a typo.
const fuzzy = archive.search(JSON.stringify({
  query: "great comprmise",
  limit: 5,
  fuzzy_distance: 2,
}));
assert(fuzzy.length > 0, "fuzzy search recovers 'comprmise' typo");

// 3. Snippet has highlight ranges.
const snippetHits = archive.search(JSON.stringify({
  query: "religion establishment",
  limit: 1,
  snippet_window: 240,
}));
assert(snippetHits.length > 0, "religion query yields a hit");
assert(snippetHits[0].snippet?.text?.length > 0, "hit carries a snippet");
assert(snippetHits[0].snippet.highlights.length > 0, "snippet has highlight ranges");

// 4. Suggest works.
const suggested = archive.suggest("ratif", 5);
assert(suggested.length > 0, `suggest('ratif') returns ${suggested.length} terms`);
assert(suggested.includes("ratification"), "suggest includes 'ratification'");

// 5. Fuzzy term lookup.
const fuzzyTerms = archive.fuzzy_terms("federlism", 2, 5);
assert(fuzzyTerms.length > 0, "fuzzy_terms returns alternatives for 'federlism'");
assert(
  fuzzyTerms.some(([t]) => t === "federalism"),
  "fuzzy_terms includes 'federalism'"
);

// 6. Process timeline lookup by phase.
const ratificationEvents = archive.process_phase("ratification");
assert(ratificationEvents.length > 0, "ratification phase has events");

// 7. Process search.
const henrySearch = archive.process_search("henry");
assert(henrySearch.length > 0, "process_search finds Henry events");

// 8. Single chunk lookup.
const firstChunkId = plain[0].chunk_id;
const chunk = archive.chunk(firstChunkId);
assert(chunk?.text?.length > 0, "chunk lookup returns text");

console.log(`\nAll smoke tests passed. Archive: ${stats.chunks} chunks, ${stats.terms} terms, ${stats.events} events.`);
