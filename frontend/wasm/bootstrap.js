// Bootstraps the WASM-backed Constitution Archive in the browser.
//
// If `pkg/constitution_wasm.js` (built by `wasm-pack`) is present, we load
// the binary archive via `fetch` and hand control to the WasmArchive class.
// Otherwise we degrade to a JSON-only timeline view so the page remains
// useful before the Rust toolchain has been run.

const STATUS_EL = document.getElementById("bootstrap");
const UI_EL = document.getElementById("ui");
const Q = document.getElementById("q");
const GO = document.getElementById("go");
const FUZZY = document.getElementById("fuzzy");
const SUGGEST = document.getElementById("suggest");
const RESULTS = document.getElementById("results");
const PHASE = document.getElementById("phase");
const PQ = document.getElementById("pq");
const EVENTS = document.getElementById("events");
const CITE_TARGET = document.getElementById("cite-target");
const CITE_SEARCH = document.getElementById("cite-search");
const CITE_GO = document.getElementById("cite-go");
const CITE_RESULTS = document.getElementById("cite-results");

const ARCHIVE_URL = "../../data/index/constitution_archive.bin";
const TIMELINE_URL = "../../data/process_timeline.json";
const PKG_URL = "./pkg/constitution_wasm.js";
const SUGGEST_DEBOUNCE_MS = 80;

function setStatus(msg, isError = false) {
  STATUS_EL.textContent = msg;
  STATUS_EL.classList.toggle("error", isError);
}

function showUI() {
  STATUS_EL.hidden = true;
  UI_EL.hidden = false;
}

function escapeHtml(s) {
  return String(s ?? "").replace(/[&<>"']/g, (c) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
  }[c]));
}

// Render `text` with `<mark>` tags around the supplied byte ranges.
// Highlights are byte-offset pairs into the *snippet* string.
function renderWithHighlights(text, highlights) {
  if (!highlights || !highlights.length) return escapeHtml(text);
  // The Rust side hands us byte offsets but JavaScript strings are UTF-16.
  // We re-walk the text in bytes by encoding once and slicing back.
  const enc = new TextEncoder();
  const dec = new TextDecoder();
  const bytes = enc.encode(text);
  let cursor = 0;
  let out = "";
  for (const [start, end] of highlights) {
    const safeStart = Math.max(cursor, Math.min(start, bytes.length));
    const safeEnd = Math.max(safeStart, Math.min(end, bytes.length));
    out += escapeHtml(dec.decode(bytes.slice(cursor, safeStart)));
    out += `<mark>${escapeHtml(dec.decode(bytes.slice(safeStart, safeEnd)))}</mark>`;
    cursor = safeEnd;
  }
  out += escapeHtml(dec.decode(bytes.slice(cursor)));
  return out;
}

function snippetHtml(snippet, fallbackPreview) {
  if (!snippet || !snippet.text) return escapeHtml(fallbackPreview || "");
  const lead = snippet.leading_ellipsis ? "…" : "";
  const trail = snippet.trailing_ellipsis ? "…" : "";
  return `${lead}${renderWithHighlights(snippet.text, snippet.highlights)}${trail}`;
}

function renderHits(hits) {
  if (!hits.length) {
    RESULTS.innerHTML = `<p class="status">No matches.</p>`;
    return;
  }
  RESULTS.innerHTML = hits
    .map(
      (h) => `
      <div class="hit" data-chunk-id="${escapeHtml(h.chunk_id)}">
        <div><strong>${escapeHtml(h.title)}</strong>
          <small>· ${escapeHtml(h.collection)} · ${escapeHtml(h.date)} · BM25 ${h.score.toFixed(2)}</small>
        </div>
        <div>${snippetHtml(h.snippet, h.preview)}</div>
        ${h.matched_terms?.length
          ? `<small>terms: ${h.matched_terms.map(escapeHtml).join(", ")}</small>`
          : ""}
        ${h.source_url ? `<small> · <a href="${h.source_url}" target="_blank" rel="noopener">source</a></small>` : ""}
        <small> · <a href="#" data-cite-from="${escapeHtml(h.chunk_id)}">show citations</a></small>
      </div>`
    )
    .join("");
}

function renderEvents(events) {
  if (!events.length) {
    EVENTS.innerHTML = `<p class="status">No events.</p>`;
    return;
  }
  EVENTS.innerHTML = events
    .map(
      (e) => `
      <div class="event">
        <div><strong>${escapeHtml(e.date)}</strong> — ${escapeHtml(e.title)}
          <small>· <span class="badge">${escapeHtml(e.phase)}</span></small>
        </div>
        <div>${escapeHtml(e.summary)}</div>
        ${e.actors?.length ? `<small>${e.actors.map(escapeHtml).join(" · ")}</small>` : ""}
      </div>`
    )
    .join("");
}

async function tryWasmBoot() {
  let mod;
  try {
    mod = await import(PKG_URL);
  } catch (_) {
    return null;
  }
  // wasm-bindgen 0.2.121+ prefers an options object; the URL is resolved
  // by the browser and the .wasm file is fetched with the right MIME type.
  await mod.default({ module_or_path: new URL("./pkg/constitution_wasm_bg.wasm", import.meta.url) });

  setStatus("Fetching binary archive…");
  const buf = await fetch(ARCHIVE_URL).then((r) => {
    if (!r.ok) throw new Error(`archive ${r.status}`);
    return r.arrayBuffer();
  });
  const archive = new mod.WasmArchive(new Uint8Array(buf));
  return { archive, mode: "wasm" };
}

async function fallbackBoot() {
  setStatus("WASM bundle not built — falling back to JSON timeline view.");
  const events = await fetch(TIMELINE_URL).then((r) => r.json());
  return { mode: "fallback", timeline: events };
}

function debounce(fn, ms) {
  let t;
  return (...args) => {
    clearTimeout(t);
    t = setTimeout(() => fn(...args), ms);
  };
}

function lastToken(value) {
  const parts = value.split(/\s+/);
  return parts[parts.length - 1] || "";
}

function replaceLastToken(value, replacement) {
  const parts = value.split(/\s+/);
  parts[parts.length - 1] = replacement;
  return parts.join(" ");
}

function wireSuggest(archive) {
  let activeIdx = -1;
  let currentTerms = [];
  const closeSuggest = () => {
    SUGGEST.hidden = true;
    SUGGEST.innerHTML = "";
    activeIdx = -1;
    currentTerms = [];
  };
  const renderSuggest = (terms) => {
    if (!terms.length) {
      closeSuggest();
      return;
    }
    currentTerms = terms;
    activeIdx = -1;
    SUGGEST.hidden = false;
    SUGGEST.innerHTML = terms.map((t) => `<div>${escapeHtml(t)}</div>`).join("");
    SUGGEST.querySelectorAll("div").forEach((el, i) => {
      el.addEventListener("mousedown", (ev) => {
        ev.preventDefault();
        Q.value = replaceLastToken(Q.value, currentTerms[i]) + " ";
        Q.focus();
        closeSuggest();
      });
    });
  };

  const refresh = debounce(() => {
    const tok = lastToken(Q.value).toLowerCase();
    if (tok.length < 2) return closeSuggest();
    try {
      const exact = archive.suggest(tok, 8);
      if (exact.length) return renderSuggest(exact);
      // Fall back to fuzzy if no prefix match.
      const fuzzy = archive.fuzzy_terms(tok, 2, 8).map(([t]) => t);
      renderSuggest(fuzzy);
    } catch (_) {
      closeSuggest();
    }
  }, SUGGEST_DEBOUNCE_MS);

  Q.addEventListener("input", refresh);
  Q.addEventListener("blur", () => setTimeout(closeSuggest, 100));
  Q.addEventListener("keydown", (ev) => {
    if (SUGGEST.hidden) return;
    const items = SUGGEST.querySelectorAll("div");
    if (ev.key === "ArrowDown") {
      ev.preventDefault();
      activeIdx = Math.min(items.length - 1, activeIdx + 1);
    } else if (ev.key === "ArrowUp") {
      ev.preventDefault();
      activeIdx = Math.max(0, activeIdx - 1);
    } else if (ev.key === "Tab" || ev.key === "Enter") {
      if (activeIdx >= 0 && currentTerms[activeIdx]) {
        ev.preventDefault();
        Q.value = replaceLastToken(Q.value, currentTerms[activeIdx]) + " ";
        closeSuggest();
        return;
      }
    } else if (ev.key === "Escape") {
      closeSuggest();
      return;
    }
    items.forEach((el, i) => el.classList.toggle("active", i === activeIdx));
  });
}

function renderCitationsFromChunk(archive, chunkId) {
  let citations;
  try {
    citations = archive.citations_from(chunkId);
  } catch (e) {
    CITE_RESULTS.innerHTML = `<p class="status error">${escapeHtml(String(e))}</p>`;
    return;
  }
  if (!citations.length) {
    CITE_RESULTS.innerHTML = `<p class="status">No outgoing citations from <code>${escapeHtml(chunkId)}</code>.</p>`;
    return;
  }
  // Group by target so the same target collapses to one row.
  const byKey = new Map();
  for (const c of citations) {
    const key = `${c.target.kind}:${c.target.id}`;
    if (!byKey.has(key)) byKey.set(key, { target: key, examples: [] });
    byKey.get(key).examples.push(c.matched_text);
  }
  const rows = [...byKey.values()];
  CITE_RESULTS.innerHTML = `
    <p class="status">Outgoing citations from <code>${escapeHtml(chunkId)}</code> · ${citations.length} total · ${rows.length} unique targets.</p>
    ${rows.map((r) => `
      <div class="hit">
        <div><strong>${escapeHtml(r.target)}</strong>
          <small>· ${r.examples.length} occurrence${r.examples.length === 1 ? "" : "s"}</small>
        </div>
        <small>${r.examples.slice(0, 4).map(escapeHtml).join(" · ")}</small>
        <small> · <a href="#" data-cite-target="${escapeHtml(r.target)}">show chunks that cite this target</a></small>
      </div>`).join("")}`;
}

function renderCitedBy(archive, targetKey) {
  let pairs;
  try {
    pairs = archive.cited_by(targetKey);
  } catch (e) {
    CITE_RESULTS.innerHTML = `<p class="status error">${escapeHtml(String(e))}</p>`;
    return;
  }
  if (!pairs.length) {
    CITE_RESULTS.innerHTML = `<p class="status">No chunks cite <code>${escapeHtml(targetKey)}</code>.</p>`;
    return;
  }
  pairs.sort((a, b) => a.chunk.date.localeCompare(b.chunk.date));
  CITE_RESULTS.innerHTML = `
    <p class="status">${pairs.length} chunk${pairs.length === 1 ? "" : "s"} cite <code>${escapeHtml(targetKey)}</code>:</p>
    ${pairs.slice(0, 80).map(({ chunk, citation }) => `
      <div class="hit">
        <div><strong>${escapeHtml(chunk.title)}</strong>
          <small>· ${escapeHtml(chunk.source_collection)} · ${escapeHtml(chunk.date)}</small>
        </div>
        <small>matched: <em>${escapeHtml(citation.matched_text)}</em></small>
      </div>`).join("")}`;
}

function populateCitationDropdown(archive) {
  const top = archive.top_citation_targets(40);
  CITE_TARGET.innerHTML =
    `<option value="">— pick a target —</option>` +
    top
      .map(([key, count]) => `<option value="${escapeHtml(key)}">${escapeHtml(key)} (${count})</option>`)
      .join("");
}

function wireWasm(archive) {
  const runSearch = () => {
    const req = JSON.stringify({
      query: Q.value,
      limit: 25,
      fuzzy_distance: FUZZY.checked ? 2 : 0,
      snippet_window: 240,
    });
    try {
      renderHits(archive.search(req));
    } catch (e) {
      RESULTS.innerHTML = `<p class="status error">${escapeHtml(String(e))}</p>`;
    }
  };
  const runEvents = () => {
    let events;
    if (PHASE.value) {
      events = archive.process_phase(PHASE.value);
    } else if (PQ.value.trim()) {
      events = archive.process_search(PQ.value.trim());
    } else {
      events = archive.timeline().events;
    }
    renderEvents(events);
  };

  GO.addEventListener("click", runSearch);
  Q.addEventListener("keydown", (ev) => {
    // Enter triggers search only when no autocomplete entry is highlighted.
    if (ev.key === "Enter" && SUGGEST.hidden) runSearch();
  });
  PHASE.addEventListener("change", runEvents);
  PQ.addEventListener("input", runEvents);
  wireSuggest(archive);
  runEvents();

  // Citation panel.
  populateCitationDropdown(archive);
  const runCiteTarget = () => {
    const key = CITE_SEARCH.value.trim() || CITE_TARGET.value;
    if (!key) {
      CITE_RESULTS.innerHTML = `<p class="status">Pick a target above to see which chunks cite it.</p>`;
      return;
    }
    renderCitedBy(archive, key);
  };
  CITE_GO.addEventListener("click", runCiteTarget);
  CITE_TARGET.addEventListener("change", runCiteTarget);
  CITE_SEARCH.addEventListener("keydown", (ev) => {
    if (ev.key === "Enter") runCiteTarget();
  });

  // Inline "show citations" link on each search hit.
  RESULTS.addEventListener("click", (ev) => {
    const a = ev.target.closest?.("[data-cite-from]");
    if (a) {
      ev.preventDefault();
      renderCitationsFromChunk(archive, a.dataset.citeFrom);
      CITE_RESULTS.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
  });
  // Inline "show chunks that cite this target" link inside the cite panel.
  CITE_RESULTS.addEventListener("click", (ev) => {
    const a = ev.target.closest?.("[data-cite-target]");
    if (a) {
      ev.preventDefault();
      CITE_SEARCH.value = a.dataset.citeTarget;
      renderCitedBy(archive, a.dataset.citeTarget);
    }
  });

  const stats = archive.stats();
  setStatus(
    `WASM archive ready — ${stats.chunks.toLocaleString()} chunks, ` +
    `${stats.terms.toLocaleString()} terms, ${stats.events} timeline events.`
  );
}

function wireFallback(state) {
  RESULTS.innerHTML = `
    <p class="status">Run <code>./scripts/build_wasm.sh</code> to enable
    full-text search, fuzzy matching, snippets, autocomplete, and the
    citation graph. The timeline view at right works without it.</p>`;
  GO.disabled = true;
  Q.disabled = true;
  FUZZY.disabled = true;
  CITE_GO.disabled = true;
  CITE_SEARCH.disabled = true;
  CITE_TARGET.disabled = true;
  CITE_RESULTS.innerHTML = `<p class="status">Citation graph requires the WASM bundle.</p>`;

  const runEvents = () => {
    let events = state.timeline.slice();
    if (PHASE.value) events = events.filter((e) => e.phase === PHASE.value);
    if (PQ.value.trim()) {
      const needle = PQ.value.trim().toLowerCase();
      events = events.filter((e) =>
        e.title.toLowerCase().includes(needle)
        || e.summary.toLowerCase().includes(needle)
        || (e.actors || []).some((a) => a.toLowerCase().includes(needle))
      );
    }
    events.sort((a, b) => a.date.localeCompare(b.date));
    renderEvents(events);
  };
  PHASE.addEventListener("change", runEvents);
  PQ.addEventListener("input", runEvents);
  runEvents();
}

(async () => {
  try {
    const wasm = await tryWasmBoot();
    if (wasm) {
      wireWasm(wasm.archive);
    } else {
      const state = await fallbackBoot();
      wireFallback(state);
    }
    showUI();
  } catch (e) {
    setStatus(`Boot failed: ${e}`, true);
  }
})();
