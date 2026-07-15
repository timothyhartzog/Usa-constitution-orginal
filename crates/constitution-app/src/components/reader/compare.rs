//! Side-by-side compare view at `/compare/:a/:b`. Renders two chunks in
//! parallel columns with a shared header so users can read related
//! passages together — useful for matching world constitutions against
//! the U.S. Bill of Rights, or Federalist vs Anti-Federalist treatments
//! of the same clause.

use dioxus::prelude::*;

use crate::components::shared::{LoadingSpinner, PermalinkButton};
use crate::router::Route;
use crate::state::use_archive;
use similar::{ChangeTag, TextDiff};

#[component]
pub fn ComparePage(a: String, b: String) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading...".to_string() } };
    }

    let chunk_a = state.chunk(&a);
    let chunk_b = state.chunk(&b);
    let mut show_diff = use_signal(|| false);

    rsx! {
        div { class: "page compare-page",
            header { class: "page-header",
                div { class: "page-header-row",
                    div {
                        h2 { "Compare passages" }
                        p { class: "page-subtitle",
                            "Two chunks side by side."
                        }
                    }
                    div { class: "header-actions", style: "display: flex; gap: 1rem;",
                        button {
                            class: "btn small-cap-button",
                            onclick: move |_| {
                                let current = *show_diff.read();
                                show_diff.set(!current);
                            },
                            if *show_diff.read() { "Hide Diff" } else { "Show Diff" }
                        }
                        PermalinkButton { label: Some("Share compare".to_string()) }
                    }
                }
            }
            if *show_diff.read() && chunk_a.is_some() && chunk_b.is_some() {
                {
                    let ca = chunk_a.as_ref().unwrap();
                    let cb = chunk_b.as_ref().unwrap();
                    let diff = TextDiff::from_words(&ca.text, &cb.text);

                    rsx! {
                        div { class: "diff-view", style: "padding: 1rem; background: #fff; border: 1px solid #ccc; font-family: monospace; white-space: pre-wrap;",
                            for change in diff.iter_all_changes() {
                                {
                                    let (bg, color) = match change.tag() {
                                        ChangeTag::Delete => ("#fee2e2", "#991b1b"),
                                        ChangeTag::Insert => ("#dcfce7", "#166534"),
                                        ChangeTag::Equal => ("transparent", "inherit"),
                                    };
                                    let sign = match change.tag() {
                                        ChangeTag::Delete => "-",
                                        ChangeTag::Insert => "+",
                                        ChangeTag::Equal => " ",
                                    };
                                    rsx! {
                                        span { style: "background-color: {bg}; color: {color};",
                                            "{change}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "compare-columns",
                    CompareColumn { chunk: chunk_a.clone(), slot_label: "A".to_string(), other_id: b.clone() }
                    CompareColumn { chunk: chunk_b.clone(), slot_label: "B".to_string(), other_id: a.clone() }
                }
            }
        }
    }
}

#[component]
fn CompareColumn(
    chunk: Option<constitution_archive::Chunk>,
    slot_label: String,
    other_id: String,
) -> Element {
    match chunk {
        Some(chunk) => rsx! {
            article { class: "compare-col",
                div { class: "compare-col-header",
                    span { class: "compare-slot-label", "{slot_label}" }
                    h3 { class: "compare-title", "{chunk.title}" }
                    div { class: "compare-meta",
                        if !chunk.author.is_empty() {
                            span { class: "compare-meta-item", "{chunk.author}" }
                        }
                        if !chunk.date.is_empty() {
                            span { class: "compare-meta-item", "{chunk.date}" }
                        }
                        span { class: "compare-meta-item", "{chunk.source_collection}" }
                    }
                    div { class: "compare-col-actions",
                        Link {
                            to: Route::DocumentPage { id: chunk.chunk_id.clone() },
                            class: "compare-action",
                            "Open full document ->"
                        }
                        Link {
                            to: Route::ComparePage { a: other_id.clone(), b: chunk.chunk_id.clone() },
                            class: "compare-action",
                            "Swap"
                        }
                    }
                }
                pre { class: "compare-text", "{chunk.text}" }
            }
        },
        None => rsx! {
            article { class: "compare-col compare-col-missing",
                div { class: "compare-col-header",
                    span { class: "compare-slot-label", "{slot_label}" }
                    h3 { class: "compare-title", "Passage not found" }
                }
                p { class: "compare-missing",
                    "The requested chunk is not in the loaded archive."
                }
            }
        },
    }
}
