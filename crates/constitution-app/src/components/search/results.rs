use dioxus::prelude::*;

use crate::components::shared::EmptyState;
use crate::router::Route;
use crate::state::{use_archive, use_search_state};

#[component]
pub fn SearchResults() -> Element {
    let archive_state = use_archive();
    let search_state = use_search_state();
    let ss = search_state.read();

    if ss.query.is_empty() {
        return rsx! {
            EmptyState {
                title: "Start searching".to_string(),
                description: "Enter a query to search across constitutional texts.".to_string(),
            }
        };
    }

    if ss.results.is_empty() {
        return rsx! {
            EmptyState {
                title: "No results".to_string(),
                description: format!("No results found for \"{}\".", ss.query),
            }
        };
    }

    let state = archive_state.read();

    rsx! {
        div { class: "search-results",
            p { class: "results-summary",
                "{ss.total_results} results for \"{ss.query}\""
            }
            div { class: "results-list",
                for hit in ss.results.iter() {
                    {
                        let chunk = state.chunk(&hit.chunk_id);
                        rsx! {
                            div { class: "result-card", key: "{hit.chunk_id}",
                                div { class: "result-header",
                                    Link {
                                        to: Route::DocumentPage { id: hit.chunk_id.clone() },
                                        class: "result-title",
                                        {chunk.as_ref().map(|c| c.title.as_str()).unwrap_or("Unknown")}
                                    }
                                    span { class: "result-score",
                                        "{hit.score:.2}"
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
                                    }
                                }
                                if !hit.snippet.text.is_empty() {
                                    p { class: "result-snippet",
                                        dangerous_inner_html: "{hit.snippet.text}"
                                    }
                                }
                                if !hit.matched_terms.is_empty() {
                                    div { class: "result-terms",
                                        for term in hit.matched_terms.iter() {
                                            span { class: "term-tag", "{term}" }
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
