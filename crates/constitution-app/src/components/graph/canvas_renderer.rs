use constitution_archive::{CitationEdge, CitationNode};
use dioxus::prelude::*;

use super::force_layout::ForceGraph;

const KIND_COLORS: &[(&str, &str, &str)] = &[
    ("person", "#c4452f", "Founders"),
    ("clause", "#2563eb", "Constitutional clauses"),
    ("essay", "#16a34a", "Federalist & Anti-Federalist"),
];

fn kind_color(kind: &str) -> &'static str {
    KIND_COLORS
        .iter()
        .find(|(k, _, _)| *k == kind)
        .map(|(_, c, _)| *c)
        .unwrap_or("#888")
}

#[component]
pub fn GraphCanvas(
    nodes: Vec<CitationNode>,
    edges: Vec<CitationEdge>,
    selected_key: Option<String>,
    on_select: EventHandler<String>,
) -> Element {
    let width = 820.0_f64;
    let height = 540.0_f64;

    let mut kind_filter = use_signal(|| Option::<String>::None);

    let active_kind = kind_filter.read().clone();

    let view = constitution_archive::CitationGraphView {
        nodes: nodes
            .iter()
            .filter(|n| active_kind.as_ref().map(|k| n.kind == *k).unwrap_or(true))
            .cloned()
            .collect(),
        edges: edges.clone(),
    };

    let graph = ForceGraph::from_view(&view, width, height);

    let max_weight = graph.edges.iter().map(|e| e.weight).max().unwrap_or(1) as f64;

    // Determine which nodes are neighbors of the selected node
    let neighbor_set: std::collections::HashSet<usize> = if let Some(ref sel) = selected_key {
        let sel_idx = graph.nodes.iter().position(|n| n.key == *sel);
        if let Some(idx) = sel_idx {
            graph
                .edges
                .iter()
                .filter_map(|e| {
                    if e.source == idx {
                        Some(e.target)
                    } else if e.target == idx {
                        Some(e.source)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            std::collections::HashSet::new()
        }
    } else {
        std::collections::HashSet::new()
    };

    let has_selection = selected_key.is_some();

    rsx! {
        div { class: "graph-canvas",
            div { class: "graph-toolbar",
                div { class: "graph-toolbar-section",
                    span { class: "toolbar-label", "Filter:" }
                    button {
                        class: if active_kind.is_none() { "toolbar-btn toolbar-btn-active" } else { "toolbar-btn" },
                        onclick: move |_| kind_filter.set(None),
                        "All"
                    }
                    for &(kind, _color, _label) in KIND_COLORS {
                        {
                            let k = kind.to_string();
                            let is_active = active_kind.as_deref() == Some(kind);
                            rsx! {
                                button {
                                    class: if is_active { "toolbar-btn toolbar-btn-active" } else { "toolbar-btn" },
                                    onclick: move |_| kind_filter.set(Some(k.clone())),
                                    "{kind}"
                                }
                            }
                        }
                    }
                }
                div { class: "graph-toolbar-section",
                    span { class: "toolbar-meta",
                        "{graph.nodes.len()} nodes, {graph.edges.len()} edges"
                    }
                }
            }
            svg {
                width: "100%",
                height: "{height}",
                view_box: "0 0 {width} {height}",
                class: "citation-svg",

                // Edges
                for edge in graph.edges.iter() {
                    {
                        let s = &graph.nodes[edge.source];
                        let t = &graph.nodes[edge.target];
                        let stroke_w = 0.5 + 3.5 * (edge.weight as f64 / max_weight);
                        let edge_is_active = if let Some(ref sel) = selected_key {
                            s.key == *sel || t.key == *sel
                        } else {
                            true
                        };
                        let opacity = if has_selection && !edge_is_active { "0.08" } else { "0.4" };
                        let color = if edge_is_active && has_selection { "#28483a" } else { "#c8c0ad" };
                        rsx! {
                            line {
                                x1: "{s.x}",
                                y1: "{s.y}",
                                x2: "{t.x}",
                                y2: "{t.y}",
                                stroke: "{color}",
                                stroke_width: "{stroke_w}",
                                opacity: "{opacity}",
                            }
                        }
                    }
                }

                // Nodes
                for (i, node) in graph.nodes.iter().enumerate() {
                    {
                        let color = kind_color(&node.kind);
                        let is_selected = selected_key.as_deref() == Some(&node.key);
                        let is_neighbor = neighbor_set.contains(&i);
                        let is_focused = is_selected || is_neighbor;
                        let stroke = if is_selected { "#000" } else { "#fff" };
                        let stroke_w = if is_selected { 3.0 } else { 1.5 };
                        let opacity = if has_selection && !is_focused { "0.25" } else { "1" };
                        let key = node.key.clone();
                        let label = node.key.split(':').last().unwrap_or(&node.key).to_string();
                        rsx! {
                            circle {
                                cx: "{node.x}",
                                cy: "{node.y}",
                                r: "{node.radius}",
                                fill: "{color}",
                                stroke: "{stroke}",
                                stroke_width: "{stroke_w}",
                                opacity: "{opacity}",
                                cursor: "pointer",
                                onclick: move |_| on_select.call(key.clone()),
                            }
                            if node.radius > 8.0 {
                                text {
                                    x: "{node.x}",
                                    y: "{node.y + node.radius + 12.0}",
                                    text_anchor: "middle",
                                    font_size: "10",
                                    fill: "#333",
                                    opacity: "{opacity}",
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }

            // Legend
            div { class: "graph-legend",
                for &(_kind, color, label) in KIND_COLORS {
                    span { class: "legend-item",
                        span {
                            class: "legend-dot",
                            style: "background: {color};",
                        }
                        "{label}"
                    }
                }
                if has_selection {
                    span { class: "legend-hint", "Selection highlighted." }
                }
            }
        }
    }
}
