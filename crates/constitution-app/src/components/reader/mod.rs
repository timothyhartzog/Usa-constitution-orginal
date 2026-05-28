mod annotated_text;
mod clause_popover;
mod widgets;

use dioxus::prelude::*;

use crate::components::shared::LoadingSpinner;
use crate::router::Route;
use crate::state::{use_archive, use_selection, SelectionKind};
use annotated_text::AnnotatedText;

pub use widgets::{ClauseComparator, MiniGraph, SearchWidget, StatWidget};

#[component]
pub fn DocumentPage(id: String) -> Element {
    let archive_state = use_archive();
    let mut selection = use_selection();
    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading...".to_string() } };
    }

    let chunk = state.chunk(&id);
    let archive = state.archive.as_ref();

    // Sibling chunks from the same document
    let siblings: Vec<constitution_archive::Chunk> = if let (Some(ref ch), Some(arc)) = (&chunk, archive) {
        arc.chunks()
            .iter()
            .filter(|c| c.document_id == ch.document_id && c.chunk_id != ch.chunk_id)
            .take(8)
            .cloned()
            .collect()
    } else {
        Vec::new()
    };

    // Process events that reference this chunk
    let events: Vec<_> = if let (Some(ref ch), Some(arc)) = (&chunk, archive) {
        arc.events_for_chunk(&ch.chunk_id)
            .into_iter()
            .cloned()
            .collect()
    } else {
        Vec::new()
    };

    // On mount, set selection state to this chunk's collection for cross-view coordination
    let id_for_effect = id.clone();
    use_effect(move || {
        selection.set(crate::state::SelectionState {
            kind: SelectionKind::Chunk(id_for_effect.clone()),
        });
    });

    match chunk {
        Some(chunk) => rsx! {
            div { class: "page document-page",
                header { class: "page-header",
                    div { class: "doc-breadcrumb",
                        Link { to: Route::SearchPage {}, "Search" }
                        span { class: "breadcrumb-sep", " / " }
                        span { class: "breadcrumb-collection", "{chunk.source_collection}" }
                    }
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
                        span { class: "tag-group-label", "Issues:" }
                        for tag in chunk.issue_tags.iter() {
                            span { class: "tag issue-tag", "{tag}" }
                        }
                    }
                }
                if !chunk.constitutional_clause_tags.is_empty() {
                    div { class: "document-tags",
                        span { class: "tag-group-label", "Clauses:" }
                        for tag in chunk.constitutional_clause_tags.iter() {
                            span { class: "tag clause-tag", "{tag}" }
                        }
                    }
                }
                section { class: "document-text",
                    AnnotatedText { text: chunk.text.clone(), chunk_id: chunk.chunk_id.clone() }
                }

                if !events.is_empty() {
                    section { class: "doc-related doc-related-events",
                        h3 { "Timeline events about this passage" }
                        div { class: "related-events-list",
                            for event in events.iter() {
                                div { class: "related-event-card",
                                    div { class: "event-header",
                                        span { class: "event-date", "{event.date}" }
                                        strong { class: "event-title", "{event.title}" }
                                    }
                                    p { class: "event-summary", "{event.summary}" }
                                }
                            }
                        }
                    }
                }

                if !siblings.is_empty() {
                    section { class: "doc-related",
                        h3 { "More from {chunk.title}" }
                        div { class: "siblings-grid",
                            for sib in siblings.iter() {
                                Link {
                                    to: Route::DocumentPage { id: sib.chunk_id.clone() },
                                    class: "sibling-card",
                                    div { class: "sibling-id", "{sib.chunk_id}" }
                                    p { class: "sibling-preview", "{sib.ensured_preview()}" }
                                }
                            }
                        }
                    }
                }
            }
        },
        None => rsx! {
            div { class: "page",
                h2 { "Document not found" }
                p { "Chunk \"{id}\" was not found in the archive." }
                Link { to: Route::SearchPage {},
                    "Back to Search"
                }
            }
        },
    }
}
