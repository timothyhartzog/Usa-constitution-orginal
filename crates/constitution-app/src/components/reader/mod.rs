mod annotated_text;
mod clause_popover;
mod widgets;

use dioxus::prelude::*;

use crate::components::shared::LoadingSpinner;
use crate::state::use_archive;
use annotated_text::AnnotatedText;

#[component]
pub fn DocumentPage(id: String) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading...".to_string() } };
    }

    let chunk = state.chunk(&id);

    match chunk {
        Some(chunk) => rsx! {
            div { class: "page document-page",
                header { class: "page-header",
                    h2 { "{chunk.title}" }
                    div { class: "document-meta",
                        if !chunk.author.is_empty() {
                            span { class: "meta-author", "by {chunk.author}" }
                        }
                        if !chunk.date.is_empty() {
                            span { class: "meta-date", "{chunk.date}" }
                        }
                        span { class: "meta-collection", "{chunk.source_collection}" }
                        span { class: "meta-words", "{chunk.word_count} words" }
                    }
                }
                if !chunk.issue_tags.is_empty() {
                    div { class: "document-tags",
                        for tag in chunk.issue_tags.iter() {
                            span { class: "tag issue-tag", "{tag}" }
                        }
                    }
                }
                if !chunk.constitutional_clause_tags.is_empty() {
                    div { class: "document-tags",
                        for tag in chunk.constitutional_clause_tags.iter() {
                            span { class: "tag clause-tag", "{tag}" }
                        }
                    }
                }
                section { class: "document-text",
                    AnnotatedText { text: chunk.text.clone(), chunk_id: chunk.chunk_id.clone() }
                }
            }
        },
        None => rsx! {
            div { class: "page",
                h2 { "Document not found" }
                p { "Chunk \"{id}\" was not found in the archive." }
                Link { to: crate::router::Route::SearchPage {},
                    "Back to Search"
                }
            }
        },
    }
}
