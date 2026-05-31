mod annotated_text;
mod annotations;
mod cite_modal;
mod clause_popover;
mod compare;
mod widgets;

use dioxus::prelude::*;

use crate::components::shared::{LoadingSpinner, PermalinkButton};
use crate::export;
use crate::router::Route;
use crate::state::{use_archive, use_selection, use_user_data, HistoryEntry, SelectionKind};
use annotated_text::AnnotatedText;
use annotations::AnnotationsPanel;
use cite_modal::CiteModal;

pub use compare::ComparePage;
pub use widgets::{ClauseComparator, MiniGraph, SearchWidget, StatWidget};

#[component]
pub fn DocumentPage(id: String) -> Element {
    let archive_state = use_archive();
    let mut selection = use_selection();
    let mut user_data = use_user_data();
    let mut reading_mode = use_signal(|| false);
    let mut cite_open = use_signal(|| false);
    let mut compare_open = use_signal(|| false);
    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading...".to_string() } };
    }

    let chunk = state.chunk(&id);
    let archive = state.archive.as_ref();
    let is_bookmarked = user_data.read().is_bookmarked(&id);

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

    // On mount, set selection state and record the visit in history.
    let id_for_effect = id.clone();
    let chunk_for_effect = chunk.clone();
    use_effect(move || {
        selection.set(crate::state::SelectionState {
            kind: SelectionKind::Chunk(id_for_effect.clone()),
        });
        if let Some(ref c) = chunk_for_effect {
            let entry = HistoryEntry {
                chunk_id: c.chunk_id.clone(),
                title: c.title.clone(),
                collection: c.source_collection.clone(),
            };
            {
                let mut u = user_data.write();
                u.push_history(entry);
            }
            crate::persist_user_data(&user_data.read());
        }
    });

    match chunk {
        Some(chunk) => rsx! {
            div { class: if *reading_mode.read() { "page document-page reading-mode" } else { "page document-page" },
                header { class: "page-header",
                    div { class: "doc-breadcrumb",
                        Link { to: Route::SearchPage {}, "Search" }
                        span { class: "breadcrumb-sep", " / " }
                        span { class: "breadcrumb-collection", "{chunk.source_collection}" }
                    }
                    div { class: "doc-title-row",
                        h2 { class: "doc-title", "{chunk.title}" }
                        div { class: "doc-title-actions",
                            button {
                                class: "bookmark-btn",
                                title: "Toggle distraction-free reading mode",
                                aria_pressed: if *reading_mode.read() { "true" } else { "false" },
                                onclick: move |_| {
                                    let cur = *reading_mode.read();
                                    reading_mode.set(!cur);
                                },
                                if *reading_mode.read() { "Exit reading mode" } else { "Reading mode" }
                            }
                            button {
                                class: "bookmark-btn",
                                aria_label: "Cite this passage",
                                onclick: move |_| cite_open.set(true),
                                "Cite"
                            }
                            button {
                                class: "bookmark-btn",
                                aria_label: "Open in compare view",
                                onclick: move |_| compare_open.set(true),
                                "Compare with..."
                            }
                        {
                            let entry = HistoryEntry {
                                chunk_id: chunk.chunk_id.clone(),
                                title: chunk.title.clone(),
                                collection: chunk.source_collection.clone(),
                            };
                            let label = if is_bookmarked { "★ Bookmarked" } else { "☆ Bookmark" };
                            let pressed = if is_bookmarked { "true" } else { "false" };
                            rsx! {
                                button {
                                    class: if is_bookmarked { "bookmark-btn bookmark-btn-active" } else { "bookmark-btn" },
                                    aria_pressed: "{pressed}",
                                    aria_label: "Toggle bookmark",
                                    onclick: move |_| {
                                        {
                                            let mut u = user_data.write();
                                            u.toggle_bookmark(entry.clone());
                                        }
                                        crate::persist_user_data(&user_data.read());
                                    },
                                    "{label}"
                                }
                            }
                        }
                        }
                    }
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

                section { class: "document-toolbar",
                    PermalinkButton { label: Some("Share passage".to_string()) }
                    {
                        let chunk_for_md = chunk.clone();
                        let filename = format!("{}.md", chunk.chunk_id);
                        rsx! {
                            button {
                                class: "btn btn-ghost",
                                title: "Download this passage as Markdown",
                                onclick: move |_| {
                                    let md = export::chunk_markdown(&chunk_for_md);
                                    let _ = export::download(&filename, "text/markdown", &md);
                                },
                                "↓ Download as Markdown"
                            }
                        }
                    }
                }

                AnnotationsPanel {
                    chunk_id: chunk.chunk_id.clone(),
                    chunk_title: chunk.title.clone(),
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

                if *cite_open.read() {
                    CiteModal {
                        chunk: chunk.clone(),
                        on_close: move |_| cite_open.set(false),
                    }
                }
                if *compare_open.read() {
                    ComparePicker {
                        from_chunk_id: chunk.chunk_id.clone(),
                        on_close: move |_| compare_open.set(false),
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

/// Modal that lets the user pick the second passage to compare against.
/// Sources are: bookmarks, recent history, and any chunk_id typed
/// directly into a free-form input.
#[component]
fn ComparePicker(from_chunk_id: String, on_close: EventHandler<()>) -> Element {
    let user_data = use_user_data();
    let mut manual = use_signal(String::new);
    let navigator = use_navigator();

    let data = user_data.read();
    let bookmarks = data.bookmarks.clone();
    let history = data.history.clone();
    drop(data);

    let from_for_filter = from_chunk_id.clone();
    let from_for_closure = from_chunk_id.clone();
    let go = move |target_id: String| {
        if !target_id.is_empty() && target_id != from_for_closure {
            on_close.call(());
            navigator.push(Route::ComparePage {
                a: from_for_closure.clone(),
                b: target_id,
            });
        }
    };

    rsx! {
        div {
            class: "modal-overlay",
            role: "dialog",
            aria_modal: "true",
            aria_label: "Open in compare view",
            onclick: move |_| on_close.call(()),
            div { class: "modal-content compare-picker",
                onclick: move |e| e.stop_propagation(),
                div { class: "modal-header",
                    h3 { "Compare with..." }
                    button {
                        class: "modal-close",
                        aria_label: "Close",
                        onclick: move |_| on_close.call(()),
                        "x"
                    }
                }
                div { class: "modal-body",
                    p { class: "compare-picker-hint",
                        "Pick a second passage to open side by side with this one."
                    }
                    if !bookmarks.is_empty() {
                        div { class: "compare-picker-section",
                            h4 { "From your bookmarks" }
                            for b in bookmarks.iter().take(8) {
                                {
                                    let id = b.chunk_id.clone();
                                    let go = go.clone();
                                    rsx! {
                                        button {
                                            class: "compare-picker-item",
                                            onclick: move |_| go(id.clone()),
                                            div { class: "compare-picker-title", "{b.title}" }
                                            div { class: "compare-picker-meta", "{b.collection}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !history.is_empty() {
                        div { class: "compare-picker-section",
                            h4 { "Recently viewed" }
                            for h in history.iter().filter(|h| h.chunk_id != from_for_filter).take(8) {
                                {
                                    let id = h.chunk_id.clone();
                                    let go = go.clone();
                                    rsx! {
                                        button {
                                            class: "compare-picker-item",
                                            onclick: move |_| go(id.clone()),
                                            div { class: "compare-picker-title", "{h.title}" }
                                            div { class: "compare-picker-meta", "{h.collection}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "compare-picker-section",
                        h4 { "Or enter a chunk ID directly" }
                        div { class: "compare-picker-manual",
                            input {
                                class: "compare-picker-input",
                                r#type: "text",
                                placeholder: "e.g. us_constitution_1787_article_i_section_8_0000",
                                value: "{manual}",
                                oninput: move |e| manual.set(e.value()),
                                onkeydown: {
                                    let go = go.clone();
                                    move |e: KeyboardEvent| {
                                        if e.key() == Key::Enter {
                                            let id = manual.read().clone();
                                            if !id.is_empty() {
                                                go(id);
                                            }
                                        }
                                    }
                                },
                            }
                            button {
                                class: "btn btn-primary",
                                onclick: {
                                    let go = go.clone();
                                    move |_| {
                                        let id = manual.read().clone();
                                        if !id.is_empty() {
                                            go(id);
                                        }
                                    }
                                },
                                "Open"
                            }
                        }
                    }
                }
            }
        }
    }
}
