use dioxus::prelude::*;

use crate::components::shared::EmptyState;
use crate::router::Route;
use crate::state::{use_archive, use_search_state};

#[component]
pub fn SearchResults() -> Element {
    let archive_state = use_archive();
    let search_state = use_search_state();
    let mut view_mode = use_signal(|| ViewMode::List);

    let ss = search_state.read();

    if ss.query.is_empty() {
        return rsx! {
            EmptyState {
                title: "Start searching".to_string(),
                description: "Enter a query to search across constitutional texts and 194 world constitutions.".to_string(),
            }
        };
    }

    if ss.results.is_empty() {
        return rsx! {
            EmptyState {
                title: "No results".to_string(),
                description: format!("No results found for \"{}\". Try a different query or adjust filters.", ss.query),
            }
        };
    }

    let state = archive_state.read();

    let collection_counts: Vec<(String, usize)> = {
        let mut map = std::collections::HashMap::new();
        for hit in &ss.results {
            if let Some(chunk) = state.chunk(&hit.chunk_id) {
                *map.entry(chunk.source_collection.clone()).or_insert(0) += 1;
            }
        }
        let mut v: Vec<_> = map.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v
    };

    rsx! {
        div { class: "search-results",
            div { class: "results-header",
                p { class: "results-summary",
                    "{ss.total_results} results for \"{ss.query}\""
                }
                div { class: "view-toggle",
                    button {
                        class: if matches!(*view_mode.read(), ViewMode::List) { "toggle-btn toggle-active" } else { "toggle-btn" },
                        onclick: move |_| view_mode.set(ViewMode::List),
                        "List"
                    }
                    button {
                        class: if matches!(*view_mode.read(), ViewMode::Grouped) { "toggle-btn toggle-active" } else { "toggle-btn" },
                        onclick: move |_| view_mode.set(ViewMode::Grouped),
                        "By Collection"
                    }
                }
            }

            // Collection facets
            if !collection_counts.is_empty() {
                div { class: "collection-facets",
                    for (col, count) in collection_counts.iter() {
                        span { class: "facet-chip", "{col} ({count})" }
                    }
                }
            }

            match *view_mode.read() {
                ViewMode::List => rsx! {
                    div { class: "results-list",
                        for hit in ss.results.iter() {
                            { ResultCard(hit, &state) }
                        }
                    }
                },
                ViewMode::Grouped => rsx! {
                    div { class: "results-grouped",
                        for (col, _count) in collection_counts.iter() {
                            div { class: "result-group",
                                h4 { class: "group-title", "{col}" }
                                for hit in ss.results.iter().filter(|h| {
                                    state.chunk(&h.chunk_id)
                                        .map(|c| &c.source_collection == col)
                                        .unwrap_or(false)
                                }) {
                                    { ResultCard(hit, &state) }
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}

#[derive(Clone, Copy)]
enum ViewMode {
    List,
    Grouped,
}

#[allow(non_snake_case)]
fn ResultCard(hit: &constitution_archive::SearchHit, state: &crate::state::ArchiveState) -> Element {
    let chunk = state.chunk(&hit.chunk_id);
    let score = hit.score;
    let chunk_id = hit.chunk_id.clone();
    let snippet_text = hit.snippet.text.clone();
    let matched_terms = hit.matched_terms.clone();

    rsx! {
        div { class: "result-card", key: "{chunk_id}",
            div { class: "result-header",
                Link {
                    to: Route::DocumentPage { id: chunk_id },
                    class: "result-title",
                    {chunk.as_ref().map(|c| c.title.as_str()).unwrap_or("Unknown")}
                }
                span { class: "result-score",
                    "{score:.2}"
                }
            }
            if let Some(ref chunk) = chunk {
                div { class: "result-meta",
                    if !chunk.author.is_empty() {
                        span { class: "result-author", "{chunk.author}" }
                    }
                    if !chunk.date.is_empty() {
                        span { class: "result-date", "{chunk.date}" }
                    }
                    span { class: "result-collection", "{chunk.source_collection}" }
                    if chunk.word_count > 0 {
                        span { class: "result-words", "{chunk.word_count} words" }
                    }
                }
            }
            if !snippet_text.is_empty() {
                p { class: "result-snippet",
                    dangerous_inner_html: "{snippet_text}"
                }
            }
            if !matched_terms.is_empty() {
                div { class: "result-terms",
                    for term in matched_terms.iter() {
                        span { class: "term-tag", "{term}" }
                    }
                }
            }
        }
    }
}
