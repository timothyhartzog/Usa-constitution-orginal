use dioxus::prelude::*;

use crate::router::Route;
use crate::state::{use_archive, use_search_state};

#[component]
pub fn WorldSearchResults() -> Element {
    let archive_state = use_archive();
    let search_state = use_search_state();
    let ss = search_state.read();

    if ss.query.is_empty() {
        return rsx! { div {} };
    }

    let state = archive_state.read();

    let world_results: Vec<_> = ss
        .results
        .iter()
        .filter(|h| {
            state
                .chunk(&h.chunk_id)
                .map(|c| {
                    c.source_collection == "comparative_constitutions_world"
                        || c.source_collection == "comparative_constitutions_eu"
                })
                .unwrap_or(false)
        })
        .collect();

    if world_results.is_empty() {
        return rsx! { div {} };
    }

    let by_country: Vec<(String, Vec<_>)> = {
        let mut map: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
        for hit in &world_results {
            if let Some(chunk) = state.chunk(&hit.chunk_id) {
                let country = chunk
                    .document_id
                    .strip_prefix("world_constitution_")
                    .or_else(|| chunk.document_id.strip_prefix("eu_constitution_"))
                    .unwrap_or(&chunk.document_id)
                    .to_string();
                map.entry(country).or_default().push(hit);
            }
        }
        let mut v: Vec<_> = map.into_iter().collect();
        v.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        v
    };

    rsx! {
        div { class: "world-results-section",
            h3 { class: "section-title",
                "World Constitution Matches ({world_results.len()} results across {by_country.len()} countries)"
            }
            div { class: "world-results-grid",
                for (country, hits) in by_country.iter() {
                    div { class: "country-result-card",
                        h4 { class: "country-result-title",
                            {country.replace('_', " ")}
                            span { class: "hit-count", " ({hits.len()})" }
                        }
                        for hit in hits.iter().take(3) {
                            {
                                let chunk = state.chunk(&hit.chunk_id);
                                rsx! {
                                    div { class: "country-result-hit",
                                        Link {
                                            to: Route::DocumentPage { id: hit.chunk_id.clone() },
                                            class: "hit-link",
                                            {chunk.as_ref().map(|c| c.title.as_str()).unwrap_or("View")}
                                        }
                                        if !hit.snippet.text.is_empty() {
                                            p { class: "hit-snippet",
                                                dangerous_inner_html: "{hit.snippet.text}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if hits.len() > 3 {
                            p { class: "more-hits", "...and {hits.len() - 3} more" }
                        }
                    }
                }
            }
        }
    }
}
