//! Coordinated multi-view dashboard. Four mini-panels share a single
//! `SelectionState` so that clicking a node in the graph highlights related
//! events in the timeline, related countries on the map, and related
//! results in the search panel.

use dioxus::prelude::*;

use crate::router::Route;
use crate::state::{use_archive, use_selection, SelectionKind, SelectionState};

#[component]
pub fn CoordinatedDashboard() -> Element {
    let selection = use_selection();
    let sel = selection.read();
    let selected_label = describe_selection(&sel);

    rsx! {
        div { class: "coordinated-dashboard",
            div { class: "coord-header",
                div {
                    h3 { "Coordinated View" }
                    p { class: "coord-subtitle",
                        "Click anywhere — graph, timeline, map, results — to link the views."
                    }
                }
                div { class: "coord-selection",
                    span { class: "coord-selection-label", "Selection:" }
                    span { class: "coord-selection-value", "{selected_label}" }
                    if sel.kind != SelectionKind::None {
                        button {
                            class: "coord-clear",
                            onclick: {
                                let mut selection = selection;
                                move |_| selection.set(SelectionState::clear())
                            },
                            "Clear"
                        }
                    }
                }
            }
            div { class: "coord-grid",
                div { class: "coord-panel coord-panel-graph",
                    div { class: "coord-panel-header",
                        h4 { "Citation Graph" }
                        Link { to: Route::GraphPage {}, class: "coord-panel-more", "Expand" }
                    }
                    MiniCitationGraph {}
                }
                div { class: "coord-panel coord-panel-timeline",
                    div { class: "coord-panel-header",
                        h4 { "Timeline" }
                        Link { to: Route::TimelinePage {}, class: "coord-panel-more", "Expand" }
                    }
                    MiniTimeline {}
                }
                div { class: "coord-panel coord-panel-map",
                    div { class: "coord-panel-header",
                        h4 { "World Map" }
                        Link { to: Route::WorldPage {}, class: "coord-panel-more", "Expand" }
                    }
                    MiniMap {}
                }
                div { class: "coord-panel coord-panel-results",
                    div { class: "coord-panel-header",
                        h4 { "Related Results" }
                        Link { to: Route::SearchPage {}, class: "coord-panel-more", "Expand" }
                    }
                    MiniRelatedResults {}
                }
            }
        }
    }
}

fn describe_selection(sel: &SelectionState) -> String {
    match &sel.kind {
        SelectionKind::None => "(none)".to_string(),
        SelectionKind::Clause(k) => format!("Clause {k}"),
        SelectionKind::Person(k) => format!("Person {k}"),
        SelectionKind::Essay(k) => format!("Essay {k}"),
        SelectionKind::Country(k) => format!("Country {k}"),
        SelectionKind::Chunk(k) => format!("Chunk {k}"),
    }
}

#[component]
fn MiniCitationGraph() -> Element {
    let archive_state = use_archive();
    let mut selection = use_selection();
    let state = archive_state.read();

    let view = state.archive.as_ref().map(|a| a.citation_graph_view(20));
    let sel = selection.read();
    let selected_key = sel.target_key();

    let Some(view) = view else {
        return rsx! { p { class: "coord-empty", "Archive loading..." } };
    };

    let width = 320.0_f64;
    let height = 220.0_f64;
    let max_count = view.nodes.iter().map(|n| n.citation_count).max().unwrap_or(1) as f64;

    let cx = width / 2.0;
    let cy = height / 2.0;
    let r = f64::min(width, height) * 0.36;

    let positioned: Vec<(String, String, f64, f64, f64)> = view
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (view.nodes.len().max(1) as f64);
            let x = cx + r * angle.cos();
            let y = cy + r * angle.sin();
            let radius = 3.0 + 8.0 * (node.citation_count as f64 / max_count).sqrt();
            (node.key.clone(), node.kind.clone(), x, y, radius)
        })
        .collect();

    let key_to_pos: std::collections::HashMap<String, (f64, f64)> = positioned
        .iter()
        .map(|(k, _, x, y, _)| (k.clone(), (*x, *y)))
        .collect();

    rsx! {
        svg {
            width: "100%",
            height: "{height}",
            view_box: "0 0 {width} {height}",
            class: "coord-graph-svg",
            // Edges
            for edge in view.edges.iter() {
                {
                    let Some(&(x1, y1)) = key_to_pos.get(&edge.source) else { return rsx! {}; };
                    let Some(&(x2, y2)) = key_to_pos.get(&edge.target) else { return rsx! {}; };
                    rsx! {
                        line {
                            x1: "{x1}", y1: "{y1}",
                            x2: "{x2}", y2: "{y2}",
                            stroke: "#c8c0ad",
                            stroke_width: "0.6",
                            opacity: "0.4",
                        }
                    }
                }
            }
            // Nodes
            for (key, kind, x, y, radius) in positioned.iter() {
                {
                    let color = kind_color(kind);
                    let is_selected = selected_key.as_deref() == Some(key);
                    let stroke = if is_selected { "#000" } else { "#fff" };
                    let stroke_w = if is_selected { 2.0 } else { 1.0 };
                    let k = key.clone();
                    rsx! {
                        circle {
                            cx: "{x}", cy: "{y}", r: "{radius}",
                            fill: "{color}",
                            stroke: "{stroke}",
                            stroke_width: "{stroke_w}",
                            cursor: "pointer",
                            onclick: move |_| {
                                let kind = k.split_once(':').map(|(p, _)| p).unwrap_or("");
                                let id = k.split_once(':').map(|(_, v)| v.to_string()).unwrap_or_default();
                                let new_kind = match kind {
                                    "clause" => SelectionKind::Clause(id),
                                    "person" => SelectionKind::Person(id),
                                    "essay" => SelectionKind::Essay(id),
                                    _ => SelectionKind::None,
                                };
                                selection.set(SelectionState { kind: new_kind });
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MiniTimeline() -> Element {
    let archive_state = use_archive();
    let selection = use_selection();
    let state = archive_state.read();

    let archive = match state.archive.as_ref() {
        Some(a) => a,
        None => return rsx! { p { class: "coord-empty", "Archive loading..." } },
    };

    let phases = archive.timeline_by_phase();
    let total: usize = phases.values().map(|v| v.len()).sum();

    if total == 0 {
        return rsx! { p { class: "coord-empty", "No timeline events." } };
    }

    let sel = selection.read();
    let selected_key = sel.target_key();

    // Find events that reference the selected key
    let related_event_ids: std::collections::HashSet<String> = if let Some(ref key) = selected_key {
        let cited = archive.cited_by(key);
        let mut event_ids = std::collections::HashSet::new();
        for (chunk, _) in &cited {
            for ev in archive.events_for_chunk(&chunk.chunk_id) {
                event_ids.insert(ev.id.clone());
            }
        }
        event_ids
    } else {
        std::collections::HashSet::new()
    };

    let width = 320.0_f64;
    let height = 200.0_f64;
    let margin = 12.0_f64;
    let usable = width - 2.0 * margin;

    let mut x_offset = margin;
    let phase_layout: Vec<(&str, f64, f64, Vec<constitution_archive::ProcessEvent>)> = phases
        .iter()
        .map(|(label, events)| {
            let w = usable * (events.len() as f64 / total as f64);
            let start = x_offset;
            x_offset += w;
            (
                *label,
                start,
                w,
                events.iter().cloned().cloned().collect::<Vec<_>>(),
            )
        })
        .collect();

    rsx! {
        svg {
            width: "100%",
            height: "{height}",
            view_box: "0 0 {width} {height}",
            class: "coord-timeline-svg",
            // Phase bands
            for &(label, start, w, ref _events) in phase_layout.iter() {
                {
                    let color = phase_color(label);
                    rsx! {
                        rect {
                            x: "{start}", y: "70", width: "{w}", height: "30",
                            fill: "{color}", opacity: "0.15", rx: "3",
                        }
                    }
                }
            }
            // Center line
            line {
                x1: "{margin}", y1: "85",
                x2: "{width - margin}", y2: "85",
                stroke: "#ccc",
                stroke_width: "1",
            }
            // Event markers
            for (label, start, w, events) in phase_layout.iter() {
                {
                    let n = events.len();
                    let color = phase_color(label);
                    rsx! {
                        for (i, ev) in events.iter().enumerate() {
                            {
                                let x = if n <= 1 {
                                    start + w / 2.0
                                } else {
                                    start + 3.0 + (w - 6.0) * (i as f64 / (n - 1) as f64)
                                };
                                let is_related = related_event_ids.contains(&ev.id);
                                let radius = if is_related { 4.5 } else { 2.5 };
                                let opacity = if related_event_ids.is_empty() || is_related { "1" } else { "0.3" };
                                rsx! {
                                    circle {
                                        cx: "{x}", cy: "85", r: "{radius}",
                                        fill: "{color}",
                                        opacity: "{opacity}",
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Labels
            for &(label, start, w, _) in phase_layout.iter() {
                text {
                    x: "{start + w / 2.0}", y: "115",
                    text_anchor: "middle",
                    font_size: "8",
                    fill: "#888",
                    "{label}"
                }
            }
        }
        if !related_event_ids.is_empty() {
            p { class: "coord-callout",
                "{related_event_ids.len()} events reference the selected target"
            }
        }
    }
}

#[component]
fn MiniMap() -> Element {
    let archive_state = use_archive();
    let selection = use_selection();
    let state = archive_state.read();

    let world_meta = &state.world_meta;

    if world_meta.is_empty() {
        return rsx! { p { class: "coord-empty", "No world metadata loaded." } };
    }

    let counts_by_region: std::collections::HashMap<String, usize> = {
        let mut m = std::collections::HashMap::new();
        for ent in world_meta {
            *m.entry(ent.region.clone()).or_insert(0) += 1;
        }
        m
    };

    let sel = selection.read();
    let selected_country = match &sel.kind {
        SelectionKind::Country(c) => Some(c.clone()),
        _ => None,
    };

    let positions: &[(&str, f64, f64)] = &[
        ("Africa", 180.0, 130.0),
        ("Americas", 70.0, 110.0),
        ("Asia", 240.0, 90.0),
        ("Europe", 180.0, 60.0),
        ("Oceania", 260.0, 170.0),
    ];

    rsx! {
        svg {
            width: "100%",
            height: "200",
            view_box: "0 0 320 200",
            class: "coord-map-svg",
            rect { x: "0", y: "0", width: "320", height: "200", fill: "#f0f4f8", rx: "4" }
            for &(region, x, y) in positions {
                {
                    let count = *counts_by_region.get(region).unwrap_or(&0);
                    let r = 12.0 + (count as f64).sqrt() * 4.0;
                    let color = region_color(region);
                    let is_highlighted = selected_country
                        .as_ref()
                        .map(|c| {
                            world_meta
                                .iter()
                                .find(|m| m.country_id == *c)
                                .map(|m| m.region == region)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false);
                    let stroke = if is_highlighted { "#000" } else { "transparent" };
                    let opacity = if selected_country.is_some() {
                        if is_highlighted { 0.8 } else { 0.2 }
                    } else {
                        0.4
                    };
                    rsx! {
                        circle {
                            cx: "{x}", cy: "{y}", r: "{r}",
                            fill: "{color}",
                            stroke: "{stroke}",
                            stroke_width: "2",
                            opacity: "{opacity}",
                        }
                        text {
                            x: "{x}", y: "{y + 2.0}",
                            text_anchor: "middle",
                            font_size: "10",
                            font_weight: "600",
                            fill: "{color}",
                            "{region}"
                        }
                        text {
                            x: "{x}", y: "{y + 14.0}",
                            text_anchor: "middle",
                            font_size: "9",
                            fill: "#555",
                            "{count}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MiniRelatedResults() -> Element {
    let archive_state = use_archive();
    let selection = use_selection();
    let state = archive_state.read();
    let sel = selection.read();

    let archive = match state.archive.as_ref() {
        Some(a) => a,
        None => return rsx! { p { class: "coord-empty", "Archive loading..." } },
    };

    let results: Vec<(String, String, String)> = match &sel.kind {
        SelectionKind::None => Vec::new(),
        SelectionKind::Clause(k) => {
            let target = format!("clause:{k}");
            archive
                .cited_by(&target)
                .into_iter()
                .take(6)
                .map(|(chunk, _)| (chunk.chunk_id.clone(), chunk.title.clone(), chunk.ensured_preview()))
                .collect()
        }
        SelectionKind::Person(k) => {
            let target = format!("person:{k}");
            archive
                .cited_by(&target)
                .into_iter()
                .take(6)
                .map(|(chunk, _)| (chunk.chunk_id.clone(), chunk.title.clone(), chunk.ensured_preview()))
                .collect()
        }
        SelectionKind::Essay(k) => {
            let target = format!("essay:{k}");
            archive
                .cited_by(&target)
                .into_iter()
                .take(6)
                .map(|(chunk, _)| (chunk.chunk_id.clone(), chunk.title.clone(), chunk.ensured_preview()))
                .collect()
        }
        SelectionKind::Country(c) => {
            // Find chunks whose document_id matches the country
            archive
                .chunks()
                .iter()
                .filter(|ch| ch.document_id.contains(c.as_str()))
                .take(6)
                .map(|ch| (ch.chunk_id.clone(), ch.title.clone(), ch.ensured_preview()))
                .collect()
        }
        SelectionKind::Chunk(id) => archive
            .chunk(id)
            .ok()
            .map(|ch| (ch.chunk_id.clone(), ch.title.clone(), ch.ensured_preview()))
            .into_iter()
            .collect(),
    };

    if results.is_empty() {
        return rsx! {
            p { class: "coord-empty",
                "Pick a node in the graph, an event in the timeline, or a country on the map to see related passages."
            }
        };
    }

    rsx! {
        div { class: "coord-results-list",
            for (chunk_id, title, preview) in results.iter() {
                Link {
                    to: Route::DocumentPage { id: chunk_id.clone() },
                    class: "coord-result",
                    div { class: "coord-result-title", "{title}" }
                    div { class: "coord-result-preview", "{preview}" }
                }
            }
        }
    }
}

fn kind_color(kind: &str) -> &'static str {
    match kind {
        "person" => "#c4452f",
        "clause" => "#2563eb",
        "essay" => "#16a34a",
        _ => "#888",
    }
}

fn phase_color(label: &str) -> &'static str {
    match label {
        "Antecedents" => "#6b7280",
        "Constitutional Convention" | "Convention" => "#2563eb",
        "Ratification debate" | "Ratification" => "#16a34a",
        "Bill of Rights drafting" | "Bill of Rights Drafting" => "#d97706",
        "Bill of Rights ratification" | "Bill of Rights Ratification" => "#c4452f",
        _ => "#888",
    }
}

fn region_color(region: &str) -> &'static str {
    match region {
        "Africa" => "#c4452f",
        "Americas" => "#2563eb",
        "Asia" => "#d97706",
        "Europe" => "#16a34a",
        "Oceania" => "#7c3aed",
        _ => "#888",
    }
}
