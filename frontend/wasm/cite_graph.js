// Compact force-directed SVG renderer for the citation co-occurrence graph.
//
// No external dependencies. Layout is computed in JS over ~150 simulation
// ticks, then frozen and drawn once. For the typical N ≤ 80 nodes this
// settles in well under 100 ms even on a phone.

const KIND_COLOR = {
  person: "#c4452f",
  clause: "#2563eb",
  essay: "#16a34a",
};

const SVG_NS = "http://www.w3.org/2000/svg";

function escapeHtml(s) {
  return String(s ?? "").replace(/[&<>"']/g, (c) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
  }[c]));
}

function nodeRadius(citationCount, maxCount) {
  const min = 4, max = 22;
  if (maxCount <= 0) return min;
  return min + (max - min) * Math.sqrt(citationCount / maxCount);
}

function edgeStroke(weight, maxWeight) {
  const min = 0.5, max = 4;
  if (maxWeight <= 0) return min;
  return min + (max - min) * (weight / maxWeight);
}

/**
 * Render a force-directed layout of `view` into `host`.
 *
 * @param {HTMLElement} host    where to mount the SVG
 * @param {{nodes: Array, edges: Array}} view
 * @param {(targetKey: string) => void} onPickTarget callback when a node is clicked
 */
export function renderCitationGraph(host, view, onPickTarget) {
  host.innerHTML = "";
  if (!view.nodes?.length) {
    host.innerHTML = `<p class="status" style="padding: 1rem;">No citation targets to render.</p>`;
    return;
  }

  const width = Math.max(host.clientWidth || 800, 320);
  const height = 480;
  const cx = width / 2, cy = height / 2;
  const maxCount = view.nodes.reduce((m, n) => Math.max(m, n.citation_count), 0);
  const maxWeight = view.edges.reduce((m, e) => Math.max(m, e.weight), 0);

  // Initialise positions on a small circle to avoid degeneracy.
  const N = view.nodes.length;
  const ringR = Math.min(width, height) * 0.28;
  const sim = view.nodes.map((node, i) => ({
    node,
    x: cx + ringR * Math.cos((2 * Math.PI * i) / N),
    y: cy + ringR * Math.sin((2 * Math.PI * i) / N),
    vx: 0,
    vy: 0,
    radius: nodeRadius(node.citation_count, maxCount),
  }));
  const idx = new Map(sim.map((s, i) => [s.node.key, i]));
  const edges = view.edges
    .filter((e) => idx.has(e.source) && idx.has(e.target))
    .map((e) => ({ a: idx.get(e.source), b: idx.get(e.target), weight: e.weight }));

  // Force constants.
  const repulse = 1400;          // node-node repulsion
  const attract = 0.04;          // edge spring constant
  const targetEdgeLen = 110;
  const centerPull = 0.012;
  const damping = 0.85;
  const maxStep = 12;

  const ticks = 200;
  for (let step = 0; step < ticks; step++) {
    // Cool the spring slightly so the layout settles.
    const heat = 1 - step / ticks;

    // Repulsion: O(N²) — fine for N <= 80.
    for (let i = 0; i < sim.length; i++) {
      for (let j = i + 1; j < sim.length; j++) {
        const dx = sim[i].x - sim[j].x;
        const dy = sim[i].y - sim[j].y;
        const d2 = dx * dx + dy * dy + 0.01;
        const force = repulse / d2;
        const fx = (force * dx) / Math.sqrt(d2);
        const fy = (force * dy) / Math.sqrt(d2);
        sim[i].vx += fx * heat;
        sim[i].vy += fy * heat;
        sim[j].vx -= fx * heat;
        sim[j].vy -= fy * heat;
      }
    }

    // Edge attraction.
    for (const e of edges) {
      const a = sim[e.a];
      const b = sim[e.b];
      const dx = b.x - a.x;
      const dy = b.y - a.y;
      const d = Math.sqrt(dx * dx + dy * dy) + 0.01;
      const stretch = (d - targetEdgeLen) * attract * Math.log2(1 + e.weight);
      const fx = (stretch * dx) / d;
      const fy = (stretch * dy) / d;
      a.vx += fx;
      a.vy += fy;
      b.vx -= fx;
      b.vy -= fy;
    }

    // Pull toward center to keep things in view.
    for (const s of sim) {
      s.vx += (cx - s.x) * centerPull;
      s.vy += (cy - s.y) * centerPull;

      // Clamp velocity, integrate, damp.
      const v = Math.sqrt(s.vx * s.vx + s.vy * s.vy);
      if (v > maxStep) {
        s.vx *= maxStep / v;
        s.vy *= maxStep / v;
      }
      s.x += s.vx;
      s.y += s.vy;
      s.vx *= damping;
      s.vy *= damping;
    }
  }

  // Clip into the viewport (with a margin for the largest node radius).
  const margin = 26;
  for (const s of sim) {
    s.x = Math.max(margin, Math.min(width - margin, s.x));
    s.y = Math.max(margin, Math.min(height - margin, s.y));
  }

  // Build the SVG.
  const svg = document.createElementNS(SVG_NS, "svg");
  svg.setAttribute("viewBox", `0 0 ${width} ${height}`);
  svg.setAttribute("width", "100%");
  svg.setAttribute("height", height);
  svg.style.background = "white";
  svg.style.display = "block";

  // Edges first (drawn beneath nodes).
  const edgeGroup = document.createElementNS(SVG_NS, "g");
  edgeGroup.setAttribute("stroke", "#c0c0c8");
  edgeGroup.setAttribute("stroke-opacity", "0.7");
  edgeGroup.setAttribute("fill", "none");
  for (const e of edges) {
    const line = document.createElementNS(SVG_NS, "line");
    line.setAttribute("x1", sim[e.a].x);
    line.setAttribute("y1", sim[e.a].y);
    line.setAttribute("x2", sim[e.b].x);
    line.setAttribute("y2", sim[e.b].y);
    line.setAttribute("stroke-width", edgeStroke(e.weight, maxWeight));
    line.append(document.createElementNS(SVG_NS, "title"));
    line.lastChild.textContent =
      `${view.nodes[e.a].label} ↔ ${view.nodes[e.b].label}: weight ${e.weight}`;
    edgeGroup.appendChild(line);
  }
  svg.appendChild(edgeGroup);

  // Nodes.
  const nodeGroup = document.createElementNS(SVG_NS, "g");
  for (const s of sim) {
    const g = document.createElementNS(SVG_NS, "g");
    g.setAttribute("transform", `translate(${s.x}, ${s.y})`);
    g.style.cursor = "pointer";
    g.setAttribute("data-key", s.node.key);
    g.addEventListener("click", () => onPickTarget?.(s.node.key));

    const c = document.createElementNS(SVG_NS, "circle");
    c.setAttribute("r", s.radius);
    c.setAttribute("fill", KIND_COLOR[s.node.kind] || "#666");
    c.setAttribute("stroke", "#fff");
    c.setAttribute("stroke-width", "1.5");
    g.appendChild(c);

    const t = document.createElementNS(SVG_NS, "title");
    t.textContent = `${s.node.key} · ${s.node.citation_count} citations`;
    g.appendChild(t);

    // Label only on the larger nodes to avoid clutter.
    if (s.radius > 8) {
      const label = document.createElementNS(SVG_NS, "text");
      label.setAttribute("text-anchor", "middle");
      label.setAttribute("dy", s.radius + 11);
      label.setAttribute("font-size", "11");
      label.setAttribute("font-family", "ui-monospace, monospace");
      label.setAttribute("fill", "#222");
      label.textContent = s.node.label.length > 20
        ? s.node.label.slice(0, 19) + "…"
        : s.node.label;
      g.appendChild(label);
    }
    nodeGroup.appendChild(g);
  }
  svg.appendChild(nodeGroup);

  host.appendChild(svg);

  // Caption.
  const summary = document.createElement("p");
  summary.className = "status";
  summary.style.margin = "0.25rem 0 0";
  summary.innerHTML = `Rendered ${escapeHtml(view.nodes.length)} nodes · ${escapeHtml(view.edges.length)} edges (max weight ${escapeHtml(maxWeight)}).`;
  host.appendChild(summary);
}
