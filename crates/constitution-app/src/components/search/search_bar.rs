use constitution_archive::SearchOptions;
use dioxus::prelude::*;

use crate::state::{use_archive, use_search_state};

#[component]
pub fn SearchBar() -> Element {
    let archive_state = use_archive();
    let mut search_state = use_search_state();
    let mut query = use_signal(String::new);

    let mut do_search = move || {
        let q = query.read().clone();
        if q.trim().is_empty() {
            search_state.write().results.clear();
            search_state.write().query.clear();
            return;
        }
        let state = archive_state.read();
        let filter = search_state.read().filter.clone();
        let opts = SearchOptions {
            limit: 50,
            fuzzy_distance: 1,
            snippet_window: 240,
            ..Default::default()
        };
        let results = state.search(&q, &filter, &opts);
        let total = results.len();
        let mut ss = search_state.write();
        ss.query = q;
        ss.results = results;
        ss.total_results = total;
    };

    rsx! {
        div { class: "search-bar",
            input {
                class: "search-input",
                r#type: "text",
                placeholder: "Search constitutional debates, papers, clauses, world constitutions...",
                value: "{query}",
                oninput: move |e| query.set(e.value()),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        do_search();
                    }
                },
            }
            button {
                class: "search-button",
                onclick: move |_| do_search(),
                "Search"
            }
        }
    }
}
