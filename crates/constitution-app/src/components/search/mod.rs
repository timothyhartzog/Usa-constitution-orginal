mod search_bar;
mod filter_panel;
mod results;
mod world_results;

use dioxus::prelude::*;

use crate::components::shared::LoadingSpinner;
use crate::state::use_archive;
use search_bar::SearchBar;
use filter_panel::FilterPanel;
use results::SearchResults;
use world_results::WorldSearchResults;

#[component]
pub fn SearchPage() -> Element {
    let archive_state = use_archive();

    let state = archive_state.read();
    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading archive...".to_string() } };
    }

    if let Some(ref err) = state.error {
        return rsx! {
            div { class: "page search-page",
                header { class: "page-header",
                    h2 { "Search" }
                }
                div { class: "error-banner",
                    h3 { "Archive not loaded" }
                    p { "{err}" }
                    p { "Run `cargo run --bin build-archive` to generate the archive, then restart." }
                }
            }
        };
    }

    let stats = state.stats();

    rsx! {
        div { class: "page search-page",
            header { class: "page-header",
                h2 { "Search" }
                p { class: "page-subtitle",
                    if let Some(ref stats) = stats {
                        "Search across {stats.chunks} text chunks from {stats.documents} documents and {stats.collections} collections."
                    } else {
                        "Full-text search across U.S. founding documents and 194 world constitutions."
                    }
                }
            }
            div { class: "search-layout",
                aside { class: "search-sidebar",
                    FilterPanel {}
                }
                main { class: "search-main",
                    SearchBar {}
                    SearchResults {}
                    WorldSearchResults {}
                }
            }
        }
    }
}
