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
const RESULTS = document.getElementById("results");
const PHASE = document.getElementById("phase");
const PQ = document.getElementById("pq");
const EVENTS = document.getElementById("events");

const ARCHIVE_URL = "../../data/index/constitution_archive.bin";
const TIMELINE_URL = "../../data/process_timeline.json";
const PKG_URL = "./pkg/constitution_wasm.js";

function setStatus(msg, isError = false) {
  STATUS_EL.textContent = msg;
  STATUS_EL.classList.toggle("error", isError);
}

function showUI() {
  STATUS_EL.hidden = true;
  UI_EL.hidden = false;
}

function renderHits(hits) {
  if (!hits.length) {
    RESULTS.innerHTML = `<p class="status">No matches.</p>`;
    return;
  }
  RESULTS.innerHTML = hits
    .map(
      (h) => `
      <div class="hit">
        <div><strong>${escapeHtml(h.title)}</strong>
          <small>· ${escapeHtml(h.collection)} · ${escapeHtml(h.date)} · BM25 ${h.score.toFixed(2)}</small>
        </div>
        <div>${escapeHtml(h.preview)}</div>
        ${h.source_url ? `<small><a href="${h.source_url}" target="_blank" rel="noopener">source</a></small>` : ""}
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

function escapeHtml(s) {
  return String(s ?? "").replace(/[&<>"']/g, (c) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
  }[c]));
}

async function tryWasmBoot() {
  // Probe pkg/constitution_wasm.js — bail to fallback if missing.
  let mod;
  try {
    mod = await import(PKG_URL);
  } catch (_) {
    return null;
  }
  await mod.default();

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
  return {
    mode: "fallback",
    timeline: events,
  };
}

function wireWasm(archive) {
  const runSearch = () => {
    const req = JSON.stringify({ query: Q.value, limit: 25 });
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
  Q.addEventListener("keydown", (ev) => ev.key === "Enter" && runSearch());
  PHASE.addEventListener("change", runEvents);
  PQ.addEventListener("input", runEvents);
  runEvents();

  const stats = archive.stats();
  setStatus(
    `WASM archive ready — ${stats.chunks.toLocaleString()} chunks, ` +
    `${stats.documents} documents, ${stats.events} timeline events.`
  );
}

function wireFallback(state) {
  RESULTS.innerHTML = `
    <p class="status">Run <code>wasm-pack build</code> to enable
    full-text search. The timeline view at right works without it.</p>`;
  GO.disabled = true;
  Q.disabled = true;

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
