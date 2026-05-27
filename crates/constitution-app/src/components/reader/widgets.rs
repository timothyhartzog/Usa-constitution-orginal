use dioxus::prelude::*;

use crate::state::use_archive;

#[component]
pub fn SearchWidget(query: String, limit: Option<usize>) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();
    let max = limit.unwrap_or(5);

    let results = state.search(
        &query,
        &constitution_archive::Filter::default(),
        &constitution_archive::SearchOptions {
            limit: max,
            snippet_window: 120,
            ..Default::default()
        },
    );

    rsx! {
        div { class: "widget search-widget",
            h4 { "Search: \"{query}\"" }
            if results.is_empty() {
                p { "No results." }
            }
            for hit in results.iter() {
                div { class: "widget-result",
                    strong { "{hit.chunk_id}" }
                    if !hit.snippet.text.is_empty() {
                        p { class: "widget-snippet",
                            dangerous_inner_html: "{hit.snippet.text}"
                        }
                    }
                }
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
                span { "{stats.chunks} chunks" }
                span { " | " }
                span { "{stats.documents} documents" }
                span { " | " }
                span { "{stats.collections} collections" }
            }
        }
    } else {
        rsx! { span { "Loading stats..." } }
    }
}
