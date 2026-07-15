mod canvas_renderer;
mod force_layout;

use dioxus::prelude::*;

use crate::components::shared::LoadingSpinner;
use crate::state::{use_archive, use_selection, SelectionKind, SelectionState};

#[component]
pub fn GraphPage() -> Element {
    let archive_state = use_archive();
    let mut selection = use_selection();
    let mut top_n = use_signal(|| 40usize);

    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading archive...".to_string() } };
    }

    let graph_view = state
        .archive
        .as_ref()
        .map(|a| a.citation_graph_view(*top_n.read()));

    let sel = selection.read();
    let selected_key = sel.target_key();

    rsx! {
        div { class: "page graph-page",
            header { class: "page-header",
                h2 { "Citation Network" }
                p { class: "page-subtitle",
                    "Explore how constitutional clauses, founders, and essays reference each other."
                }
            }
            div { class: "graph-controls",
                label {
                    "Top targets: "
                    input {
                        r#type: "range",
                        min: "10",
                        max: "100",
                        value: "{top_n}",
                        oninput: move |e| {
                            if let Ok(n) = e.value().parse::<usize>() {
                                top_n.set(n);
                            }
                        },
                    }
                    span { " {top_n}" }
                }
                if selected_key.is_some() {
                    button {
                        class: "clear-selection",
                        onclick: move |_| selection.set(SelectionState::clear()),
                        "Clear Selection"
                    }
                }
            }
            div { class: "graph-layout",
                div { class: "graph-canvas-container",
                    if let Some(ref view) = graph_view {
                        canvas_renderer::GraphCanvas {
                            nodes: view.nodes.clone(),
                            edges: view.edges.clone(),
                            selected_key: selected_key.clone(),
                            on_select: move |key: String| {
                                let kind = if key.starts_with("clause:") {
                                    SelectionKind::Clause(key[7..].to_string())
                                } else if key.starts_with("person:") {
                                    SelectionKind::Person(key[7..].to_string())
                                } else if key.starts_with("essay:") {
                                    SelectionKind::Essay(key[6..].to_string())
                                } else {
                                    SelectionKind::Clause(key.clone())
                                };
                                selection.set(SelectionState { kind });
                            },
                        }
                    }
                }
                aside { class: "graph-detail",
                    if let Some(ref key) = selected_key {
                        NodeDetail { target_key: key.clone() }
                    } else {
                        p { class: "graph-hint", "Click a node to see details." }
                    }
                }
            }
        }
    }
}

#[component]
fn NodeDetail(target_key: String) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    let cited_by = state
        .archive
        .as_ref()
        .map(|a| a.cited_by(&target_key))
        .unwrap_or_default();

    rsx! {
        div { class: "node-detail",
            h3 { "{target_key}" }
            p { "{cited_by.len()} references across corpus" }
            ul { class: "node-refs",
                for (chunk, _citation) in cited_by.iter().take(20) {
                    li {
                        Link {
                            to: crate::router::Route::DocumentPage { id: chunk.chunk_id.clone() },
                            "{chunk.title}"
                        }
                        span { class: "ref-meta", " ({chunk.author}, {chunk.date})" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn KnowledgeGraphPage() -> Element {
    let archive_state = use_archive();
    let mut selected_person = use_signal(|| Option::<String>::None);
    let mut depth = use_signal(|| 2usize);
    let mut max_nodes = use_signal(|| 80usize);

    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading archive...".to_string() } };
    }

    let graph_view = state.knowledge_graph.as_ref().and_then(|kg| {
        let root = selected_person
            .read()
            .clone()
            .unwrap_or_else(|| "Washington".to_string());

        let (kg_nodes, kg_edges) = kg.ego_graph(&root, *depth.read(), *max_nodes.read());

        let nodes: Vec<constitution_archive::CitationNode> = kg_nodes
            .into_iter()
            .map(|n| constitution_archive::CitationNode {
                key: format!("person:{}", n.id),
                kind: "person".to_string(),
                label: n.id,
                citation_count: 50, // dummy size
            })
            .collect();

        let edges: Vec<constitution_archive::CitationEdge> = kg_edges
            .into_iter()
            .map(|e| constitution_archive::CitationEdge {
                source: format!("person:{}", e.source),
                target: format!("person:{}", e.target),
                weight: e.weight as u32,
            })
            .collect();

        Some(constitution_archive::CitationGraphView { nodes, edges })
    });

    let selected_key = selected_person
        .read()
        .as_ref()
        .map(|p| format!("person:{p}"));

    rsx! {
        div { class: "page graph-page",
            header { class: "page-header",
                h2 { "Historical Network" }
                p { class: "page-subtitle",
                    "Explore co-occurrences of historical figures and geopolitical entities."
                }
            }
            div { class: "graph-controls",
                label {
                    "Focal Entity: "
                    input {
                        r#type: "text",
                        placeholder: "e.g. Washington",
                        onchange: move |e| {
                            let val = e.value().trim().to_string();
                            if !val.is_empty() {
                                selected_person.set(Some(val));
                            }
                        },
                    }
                }
                label {
                    "Depth: "
                    input {
                        r#type: "range", min: "1", max: "3", value: "{depth}",
                        oninput: move |e| {
                            if let Ok(n) = e.value().parse::<usize>() { depth.set(n); }
                        },
                    }
                    span { " {depth}" }
                }
                label {
                    "Max Nodes: "
                    input {
                        r#type: "range", min: "10", max: "200", value: "{max_nodes}",
                        oninput: move |e| {
                            if let Ok(n) = e.value().parse::<usize>() { max_nodes.set(n); }
                        },
                    }
                    span { " {max_nodes}" }
                }
            }
            div { class: "graph-layout",
                div { class: "graph-canvas-container",
                    if let Some(ref view) = graph_view {
                        canvas_renderer::GraphCanvas {
                            nodes: view.nodes.clone(),
                            edges: view.edges.clone(),
                            selected_key: selected_key.clone(),
                            on_select: move |key: String| {
                                if key.starts_with("person:") {
                                    selected_person.set(Some(key[7..].to_string()));
                                }
                            },
                        }
                    } else {
                        p { "Entity not found in the knowledge graph. Try 'Washington' or 'Madison'." }
                    }
                }
                aside { class: "graph-detail",
                    if let Some(ref p) = *selected_person.read() {
                        div { class: "node-detail",
                            h3 { "{p}" }
                            if let Some(kg) = state.knowledge_graph.as_ref() {
                                p { "{kg.connections_for(p, 50).len()} direct connections." }
                            }
                        }
                    } else {
                        p { class: "graph-hint", "Click a node or search an entity to center the network." }
                    }
                }
            }
        }
    }
}
