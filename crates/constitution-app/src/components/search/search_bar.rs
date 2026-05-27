use constitution_archive::SearchOptions;
use dioxus::prelude::*;

use crate::state::{use_archive, use_search_state};

#[component]
pub fn SearchBar() -> Element {
    let archive_state = use_archive();
    let mut search_state = use_search_state();
    let mut query = use_signal(String::new);
    let mut suggestions = use_signal(Vec::<String>::new);
    let mut show_suggestions = use_signal(|| false);

    let mut do_search = move || {
        let q = query.read().clone();
        show_suggestions.set(false);
        if q.trim().is_empty() {
            let mut ss = search_state.write();
            ss.results.clear();
            ss.query.clear();
            ss.total_results = 0;
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

    let on_input = move |e: Event<FormData>| {
        let val = e.value();
        query.set(val.clone());

        let state = archive_state.read();
        if val.len() >= 2 {
            if let Some(ref archive) = state.archive {
                let sug = archive.suggest(&val, 8);
                suggestions.set(sug);
                show_suggestions.set(true);
            }
        } else {
            suggestions.set(Vec::new());
            show_suggestions.set(false);
        }
    };

    rsx! {
        div { class: "search-bar-container",
            div { class: "search-bar",
                input {
                    class: "search-input",
                    r#type: "text",
                    placeholder: "Search constitutional debates, papers, clauses, world constitutions...",
                    value: "{query}",
                    oninput: on_input,
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            do_search();
                        }
                    },
                    onfocusout: move |_| {
                        show_suggestions.set(false);
                    },
                }
                button {
                    class: "search-button",
                    onclick: move |_| do_search(),
                    "Search"
                }
            }
            if *show_suggestions.read() && !suggestions.read().is_empty() {
                div { class: "suggestions-dropdown",
                    for sug in suggestions.read().iter() {
                        {
                            let s = sug.clone();
                            rsx! {
                                button {
                                    class: "suggestion-item",
                                    onmousedown: move |_| {
                                        query.set(s.clone());
                                        show_suggestions.set(false);
                                        do_search();
                                    },
                                    "{sug}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
