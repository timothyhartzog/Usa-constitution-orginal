use constitution_archive::{Filter, FilterValue, SearchOptions};
use dioxus::prelude::*;

use crate::router::Route;
use crate::state::use_archive;

#[component]
pub fn SearchWidget(query: String, limit: Option<usize>) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();
    let max = limit.unwrap_or(5);

    let results = state.search(
        &query,
        &Filter::default(),
        &SearchOptions {
            limit: max,
            snippet_window: 140,
            ..Default::default()
        },
    );

    rsx! {
        div { class: "widget search-widget",
            div { class: "widget-header",
                span { class: "widget-kind", "Search" }
                strong { "\"{query}\"" }
                span { class: "widget-count", "{results.len()} hits" }
            }
            if results.is_empty() {
                p { class: "widget-empty", "No results." }
            }
            for hit in results.iter() {
                {
                    let chunk = state.chunk(&hit.chunk_id);
                    let title = chunk.as_ref().map(|c| c.title.clone()).unwrap_or_else(|| "Unknown".to_string());
                    let author = chunk.as_ref().map(|c| c.author.clone()).unwrap_or_default();
                    rsx! {
                        Link {
                            to: Route::DocumentPage { id: hit.chunk_id.clone() },
                            class: "widget-result",
                            div { class: "widget-result-title", "{title}" }
                            if !author.is_empty() {
                                div { class: "widget-result-author", "{author}" }
                            }
                            if !hit.snippet.text.is_empty() {
                                div { class: "widget-result-snippet",
                                    dangerous_inner_html: "{hit.snippet.text}"
                                }
                            }
                        }
                    }
                }
            }
            Link { to: Route::SearchPage {}, class: "widget-more",
                "View all results -->"
            }
        }
    }
}

#[component]
pub fn StatWidget() -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    if let Some(stats) = state.stats() {
        rsx! {
            div { class: "widget stat-widget",
                div { class: "widget-header",
                    span { class: "widget-kind", "Archive Stats" }
                }
                div { class: "widget-stats-grid",
                    div { class: "widget-stat",
                        span { class: "widget-stat-value", "{stats.chunks}" }
                        span { class: "widget-stat-label", "chunks" }
                    }
                    div { class: "widget-stat",
                        span { class: "widget-stat-value", "{stats.documents}" }
                        span { class: "widget-stat-label", "documents" }
                    }
                    div { class: "widget-stat",
                        span { class: "widget-stat-value", "{stats.collections}" }
                        span { class: "widget-stat-label", "collections" }
                    }
                    div { class: "widget-stat",
                        span { class: "widget-stat-value", "{stats.citations}" }
                        span { class: "widget-stat-label", "citations" }
                    }
                    div { class: "widget-stat",
                        span { class: "widget-stat-value", "{state.world_meta.len()}" }
                        span { class: "widget-stat-label", "world constitutions" }
                    }
                }
            }
        }
    } else {
        rsx! { span { "Loading stats..." } }
    }
}

/// Mini citation graph centered on a target key. Renders top N co-cited targets as a starburst.
#[component]
pub fn MiniGraph(target_key: String, max_links: Option<usize>) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();
    let n = max_links.unwrap_or(8);

    let archive = match state.archive.as_ref() {
        Some(a) => a,
        None => return rsx! { div { class: "widget", "Loading..." } },
    };

    let view = archive.citation_graph_view(40);
    let center_idx = view.nodes.iter().position(|n| n.key == target_key);

    let neighbors: Vec<(String, String, u32)> = if let Some(idx) = center_idx {
        let target_keys: std::collections::HashMap<usize, &str> = view
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (i, node.kind.as_str()))
            .collect();

        let key_to_idx: std::collections::HashMap<&str, usize> = view
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node.key.as_str(), i))
            .collect();

        let mut edges: Vec<_> = view
            .edges
            .iter()
            .filter_map(|e| {
                let s_idx = key_to_idx.get(e.source.as_str()).copied()?;
                let t_idx = key_to_idx.get(e.target.as_str()).copied()?;
                if s_idx == idx {
                    Some((
                        e.target.clone(),
                        target_keys.get(&t_idx).copied().unwrap_or("").to_string(),
                        e.weight,
                    ))
                } else if t_idx == idx {
                    Some((
                        e.source.clone(),
                        target_keys.get(&s_idx).copied().unwrap_or("").to_string(),
                        e.weight,
                    ))
                } else {
                    None
                }
            })
            .collect();
        edges.sort_by(|a, b| b.2.cmp(&a.2));
        edges.truncate(n);
        edges
    } else {
        Vec::new()
    };

    let center_label = target_key
        .split_once(':')
        .map(|(_, v)| v)
        .unwrap_or(&target_key);
    let center_count = archive.cited_by(&target_key).len();

    rsx! {
        div { class: "widget mini-graph-widget",
            div { class: "widget-header",
                span { class: "widget-kind", "Mini Citation Graph" }
                strong { "{center_label}" }
            }
            div { class: "mini-graph",
                svg {
                    width: "100%",
                    height: "200",
                    view_box: "0 0 320 200",
                    class: "mini-graph-svg",
                    // Center node
                    {
                        let label = center_label.to_string();
                        rsx! {
                            circle { cx: "160", cy: "100", r: "18", fill: "#28483a", }
                            text {
                                x: "160", y: "104",
                                text_anchor: "middle",
                                font_size: "11",
                                fill: "#fff",
                                font_weight: "600",
                                "{label}"
                            }
                        }
                    }
                    // Neighbor nodes arranged in a circle
                    {
                        let count = neighbors.len();
                        if count == 0 {
                            rsx! {
                                text {
                                    x: "160", y: "150",
                                    text_anchor: "middle",
                                    font_size: "11",
                                    fill: "#888",
                                    "No co-citations found"
                                }
                            }
                        } else {
                            let max_w = neighbors.iter().map(|n| n.2).max().unwrap_or(1) as f64;
                            rsx! {
                                for (i, (key, kind, weight)) in neighbors.iter().enumerate() {
                                    {
                                        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (count as f64);
                                        let r = 80.0;
                                        let x = 160.0 + r * angle.cos();
                                        let y = 100.0 + r * angle.sin();
                                        let stroke_w = 0.5 + 3.0 * (*weight as f64 / max_w);
                                        let color = match kind.as_str() {
                                            "person" => "#c4452f",
                                            "clause" => "#2563eb",
                                            "essay" => "#16a34a",
                                            _ => "#888",
                                        };
                                        let label = key.split_once(':').map(|(_, v)| v).unwrap_or(key).to_string();
                                        rsx! {
                                            line {
                                                x1: "160", y1: "100",
                                                x2: "{x}", y2: "{y}",
                                                stroke: "#c8c0ad",
                                                stroke_width: "{stroke_w}",
                                                opacity: "0.5",
                                            }
                                            circle {
                                                cx: "{x}", cy: "{y}", r: "10",
                                                fill: "{color}",
                                            }
                                            text {
                                                x: "{x}", y: "{y + 24.0}",
                                                text_anchor: "middle",
                                                font_size: "10",
                                                fill: "#333",
                                                "{label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "widget-footer",
                span { class: "widget-stat-label", "{center_count} total references" }
                Link { to: Route::GraphPage {}, class: "widget-more",
                    "Full graph -->"
                }
            }
        }
    }
}

/// Side-by-side clause comparator. Searches for a topic across multiple
/// collections and shows top results from each.
#[component]
pub fn ClauseComparator(topic: String, collections: Option<String>) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    let cols: Vec<String> = collections
        .as_ref()
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_else(|| {
            vec![
                "constitution".to_string(),
                "federalist_papers".to_string(),
                "anti_federalist".to_string(),
            ]
        });

    rsx! {
        div { class: "widget comparator-widget",
            div { class: "widget-header",
                span { class: "widget-kind", "Comparison" }
                strong { "How is \"{topic}\" treated across collections?" }
            }
            div { class: "comparator-grid",
                for col in cols.iter() {
                    {
                        let mut filter = Filter::default();
                        filter = filter.with(FilterValue::Collection(vec![col.clone()]));
                        let results = state.search(
                            &topic,
                            &filter,
                            &SearchOptions {
                                limit: 2,
                                snippet_window: 140,
                                fuzzy_distance: 0,
                                ..Default::default()
                            },
                        );
                        let col_label = col.replace('_', " ");
                        rsx! {
                            div { class: "comparator-cell",
                                h4 { class: "comparator-col", "{col_label}" }
                                if results.is_empty() {
                                    p { class: "comparator-empty", "No matches in this collection." }
                                }
                                for hit in results.iter() {
                                    {
                                        let chunk = state.chunk(&hit.chunk_id);
                                        let title = chunk.as_ref().map(|c| c.title.clone()).unwrap_or_default();
                                        rsx! {
                                            Link {
                                                to: Route::DocumentPage { id: hit.chunk_id.clone() },
                                                class: "comparator-result",
                                                div { class: "comparator-title", "{title}" }
                                                if !hit.snippet.text.is_empty() {
                                                    div {
                                                        class: "comparator-snippet",
                                                        dangerous_inner_html: "{hit.snippet.text}",
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
