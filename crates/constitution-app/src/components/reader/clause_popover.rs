use dioxus::prelude::*;

use crate::state::use_archive;

#[component]
pub fn ClausePopover(target_key: String) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    let cited_by = state
        .archive
        .as_ref()
        .map(|a| a.cited_by(&target_key))
        .unwrap_or_default();

    rsx! {
        div { class: "clause-popover",
            h4 { class: "popover-title", "{target_key}" }
            p { class: "popover-count", "{cited_by.len()} references in corpus" }
            if !cited_by.is_empty() {
                ul { class: "popover-refs",
                    for (chunk, _citation) in cited_by.iter().take(10) {
                        li {
                            strong { "{chunk.title}" }
                            span { " by {chunk.author}" }
                            span { class: "ref-date", " ({chunk.date})" }
                        }
                    }
                }
                if cited_by.len() > 10 {
                    p { class: "popover-more",
                        "...and {cited_by.len() - 10} more"
                    }
                }
            }
        }
    }
}
