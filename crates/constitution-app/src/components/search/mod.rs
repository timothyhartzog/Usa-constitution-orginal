mod search_bar;
mod filter_panel;
mod results;

use dioxus::prelude::*;

use crate::components::shared::LoadingSpinner;
use crate::state::{use_archive, use_search_state};
use search_bar::SearchBar;
use filter_panel::FilterPanel;
use results::SearchResults;

#[component]
pub fn SearchPage() -> Element {
    let archive_state = use_archive();
    let _search_state = use_search_state();

    let state = archive_state.read();
    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading archive...".to_string() } };
    }

    rsx! {
        div { class: "page search-page",
            header { class: "page-header",
                h2 { "Search" }
                p { class: "page-subtitle",
                    "Full-text search across U.S. founding documents and 194 world constitutions."
                }
            }
            div { class: "search-layout",
                aside { class: "search-sidebar",
                    FilterPanel {}
                }
                main { class: "search-main",
                    SearchBar {}
                    SearchResults {}
                }
            }
        }
    }
}
