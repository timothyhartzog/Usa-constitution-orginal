use dioxus::prelude::*;

use crate::state::{use_archive, use_selection, SelectionState};

#[component]
pub fn AnnotatedText(text: String, chunk_id: String) -> Element {
    let archive_state = use_archive();
    let mut selection = use_selection();

    let state = archive_state.read();
    let citations = state
        .archive
        .as_ref()
        .and_then(|a| a.citations_from(&chunk_id).ok())
        .unwrap_or_default();

    let has_citations = !citations.is_empty();

    rsx! {
        div { class: "annotated-text",
            if has_citations {
                div { class: "citation-sidebar",
                    h4 { "References in this passage" }
                    for citation in citations.iter() {
                        button {
                            class: "citation-link",
                            onclick: {
                                let key = citation.target.key();
                                move |_| {
                                    selection.set(SelectionState { kind: crate::state::SelectionKind::Clause(key.clone()) });
                                }
                            },
                            span { class: "citation-target", "{citation.target.key()}" }
                            span { class: "citation-matched", "\"{citation.matched_text}\"" }
                        }
                    }
                }
            }
            div { class: "text-body",
                pre { class: "document-content", "{text}" }
            }
        }
    }
}
