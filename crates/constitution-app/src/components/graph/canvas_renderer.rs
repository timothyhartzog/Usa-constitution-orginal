use constitution_archive::{CitationEdge, CitationNode};
use dioxus::prelude::*;

use super::force_layout::ForceGraph;

const KIND_COLORS: &[(&str, &str)] = &[
    ("person", "#c4452f"),
    ("clause", "#2563eb"),
    ("essay", "#16a34a"),
];

fn kind_color(kind: &str) -> &'static str {
    KIND_COLORS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, c)| *c)
        .unwrap_or("#888")
}

#[component]
pub fn GraphCanvas(
    nodes: Vec<CitationNode>,
    edges: Vec<CitationEdge>,
    selected_key: Option<String>,
    on_select: EventHandler<String>,
) -> Element {
    let width = 800.0_f64;
    let height = 520.0_f64;

    let view = constitution_archive::CitationGraphView {
        nodes: nodes.clone(),
        edges: edges.clone(),
    };
    let graph = ForceGraph::from_view(&view, width, height);

    let max_weight = graph
        .edges
        .iter()
        .map(|e| e.weight)
        .max()
        .unwrap_or(1) as f64;

    rsx! {
        div { class: "graph-canvas",
            svg {
                width: "{width}",
                height: "{height}",
                view_box: "0 0 {width} {height}",
                class: "citation-svg",

                // Edges
                for edge in graph.edges.iter() {
                    {
                        let s = &graph.nodes[edge.source];
                        let t = &graph.nodes[edge.target];
                        let stroke_w = 0.5 + 3.5 * (edge.weight as f64 / max_weight);
                        rsx! {
                            line {
                                x1: "{s.x}",
                                y1: "{s.y}",
                                x2: "{t.x}",
                                y2: "{t.y}",
                                stroke: "#c8c0ad",
                                stroke_width: "{stroke_w}",
                                opacity: "0.4",
                            }
                        }
                    }
                }

                // Nodes
                for node in graph.nodes.iter() {
                    {
                        let color = kind_color(&node.kind);
                        let is_selected = selected_key.as_deref() == Some(&node.key);
                        let stroke = if is_selected { "#000" } else { "#fff" };
                        let stroke_w = if is_selected { 3.0 } else { 1.5 };
                        let key = node.key.clone();
                        rsx! {
                            circle {
                                cx: "{node.x}",
                                cy: "{node.y}",
                                r: "{node.radius}",
                                fill: "{color}",
                                stroke: "{stroke}",
                                stroke_width: "{stroke_w}",
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
                                    {node.key.split(':').last().unwrap_or(&node.key).to_string()}
                                }
                            }
                        }
                    }
                }
            }

            // Legend
            div { class: "graph-legend",
                for &(kind, color) in KIND_COLORS {
                    span { class: "legend-item",
                        span {
                            class: "legend-dot",
                            style: "background: {color};",
                        }
                        "{kind}"
                    }
                }
            }
        }
    }
}
