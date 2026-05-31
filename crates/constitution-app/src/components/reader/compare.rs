//! Side-by-side compare view at `/compare/:a/:b`. Renders two chunks in
//! parallel columns with a shared header so users can read related
//! passages together — useful for matching world constitutions against
//! the U.S. Bill of Rights, or Federalist vs Anti-Federalist treatments
//! of the same clause.

use dioxus::prelude::*;

use crate::components::shared::{LoadingSpinner, PermalinkButton};
use crate::router::Route;
use crate::state::use_archive;

#[component]
pub fn ComparePage(a: String, b: String) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading...".to_string() } };
    }

    let chunk_a = state.chunk(&a);
    let chunk_b = state.chunk(&b);

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
                    PermalinkButton { label: Some("Share compare".to_string()) }
                }
            }
            div { class: "compare-columns",
                CompareColumn { chunk: chunk_a, slot_label: "A", other_id: b.clone() }
                CompareColumn { chunk: chunk_b, slot_label: "B", other_id: a.clone() }
            }
        }
    }
}

#[component]
fn CompareColumn(chunk: Option<constitution_archive::Chunk>, slot_label: String, other_id: String) -> Element {
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
