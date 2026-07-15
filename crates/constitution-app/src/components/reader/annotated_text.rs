use dioxus::prelude::*;

use crate::router::Route;
use crate::state::{use_archive, use_selection, SelectionKind, SelectionState};

/// A span of text in the chunk, either plain or a citation.
#[derive(Debug, Clone)]
enum TextSpan {
    Plain(String),
    Citation {
        text: String,
        target_key: String,
        kind: String,
    },
}

fn build_spans(text: &str, citations: &[CitationLite]) -> Vec<TextSpan> {
    let mut spans: Vec<TextSpan> = Vec::new();
    let mut cursor = 0usize;

    let mut sorted = citations.to_vec();
    sorted.sort_by_key(|c| c.byte_offset);

    for c in sorted {
        if c.byte_offset < cursor || c.byte_offset > text.len() {
            continue;
        }
        let match_end = c.byte_offset + c.matched_text.len();
        if match_end > text.len() {
            continue;
        }
        if !text.is_char_boundary(c.byte_offset) || !text.is_char_boundary(match_end) {
            continue;
        }
        if c.byte_offset > cursor {
            spans.push(TextSpan::Plain(text[cursor..c.byte_offset].to_string()));
        }
        spans.push(TextSpan::Citation {
            text: text[c.byte_offset..match_end].to_string(),
            target_key: c.target_key.clone(),
            kind: c.kind.clone(),
        });
        cursor = match_end;
    }
    if cursor < text.len() {
        spans.push(TextSpan::Plain(text[cursor..].to_string()));
    }
    spans
}

#[derive(Debug, Clone)]
struct CitationLite {
    byte_offset: usize,
    matched_text: String,
    target_key: String,
    kind: String,
}

#[component]
pub fn AnnotatedText(text: String, chunk_id: String) -> Element {
    let archive_state = use_archive();
    let mut selection = use_selection();
    let mut active_popover = use_signal(|| Option::<String>::None);

    let state = archive_state.read();
    let citations_lite: Vec<CitationLite> = state
        .archive
        .as_ref()
        .and_then(|a| a.citations_from(&chunk_id).ok())
        .map(|cits| {
            cits.into_iter()
                .map(|c| {
                    let key = c.target.key();
                    let kind = key
                        .split_once(':')
                        .map(|(k, _)| k.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    CitationLite {
                        byte_offset: c.byte_offset,
                        matched_text: c.matched_text.clone(),
                        target_key: key,
                        kind,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let spans = build_spans(&text, &citations_lite);

    // Group citations by target for sidebar display
    let mut by_target: std::collections::BTreeMap<String, Vec<CitationLite>> =
        std::collections::BTreeMap::new();
    for c in &citations_lite {
        by_target
            .entry(c.target_key.clone())
            .or_default()
            .push(c.clone());
    }

    rsx! {
        div { class: "annotated-text",
            div { class: "text-body",
                article { class: "document-content",
                    for (i, span) in spans.iter().enumerate() {
                        {
                            let key = format!("span-{i}");
                            match span {
                                TextSpan::Plain(s) => rsx! {
                                    span { key: "{key}", "{s}" }
                                },
                                TextSpan::Citation { text, target_key, kind } => {
                                    let tk = target_key.clone();
                                    let tk2 = target_key.clone();
                                    let css_class = format!("inline-citation citation-{kind}");
                                    rsx! {
                                        span {
                                            key: "{key}",
                                            class: "{css_class}",
                                            title: "{tk}",
                                            onclick: move |_| {
                                                let cur = active_popover.read().clone();
                                                if cur.as_deref() == Some(&tk) {
                                                    active_popover.set(None);
                                                } else {
                                                    active_popover.set(Some(tk.clone()));
                                                }
                                                let kind_str = tk.split_once(':').map(|(k, _)| k).unwrap_or("");
                                                let key_str = tk.split_once(':').map(|(_, v)| v.to_string()).unwrap_or_default();
                                                let sel_kind = match kind_str {
                                                    "clause" => SelectionKind::Clause(key_str),
                                                    "person" => SelectionKind::Person(key_str),
                                                    "essay" => SelectionKind::Essay(key_str),
                                                    _ => SelectionKind::None,
                                                };
                                                selection.set(SelectionState { kind: sel_kind });
                                            },
                                            "{text}"
                                            if active_popover.read().as_deref() == Some(&tk2) {
                                                InlinePopover { target_key: tk2 }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            aside { class: "citation-sidebar",
                h4 { "References ({by_target.len()})" }
                if by_target.is_empty() {
                    p { class: "sidebar-empty", "No citations detected in this passage." }
                }
                for (target_key, hits) in by_target.iter() {
                    {
                        let tk = target_key.clone();
                        let kind = tk.split_once(':').map(|(k, _)| k).unwrap_or("");
                        let label = tk.split_once(':').map(|(_, v)| v).unwrap_or(&tk);
                        let css_class = format!("citation-link citation-{kind}-link");
                        let hit_count = hits.len();
                        let first_match = hits.first().map(|h| h.matched_text.clone()).unwrap_or_default();
                        rsx! {
                            button {
                                class: "{css_class}",
                                onclick: {
                                    let tk = tk.clone();
                                    move |_| {
                                        let kind_str = tk.split_once(':').map(|(k, _)| k).unwrap_or("");
                                        let key_str = tk.split_once(':').map(|(_, v)| v.to_string()).unwrap_or_default();
                                        let sel_kind = match kind_str {
                                            "clause" => SelectionKind::Clause(key_str),
                                            "person" => SelectionKind::Person(key_str),
                                            "essay" => SelectionKind::Essay(key_str),
                                            _ => SelectionKind::None,
                                        };
                                        selection.set(SelectionState { kind: sel_kind });
                                    }
                                },
                                div { class: "citation-link-header",
                                    span { class: "citation-target", "{label}" }
                                    if hit_count > 1 {
                                        span { class: "citation-link-count", "x{hit_count}" }
                                    }
                                }
                                if !first_match.is_empty() {
                                    span { class: "citation-matched", "\"{first_match}\"" }
                                }
                            }
                        }
                    }
                }
                if let Some(target) = active_popover.read().clone() {
                    SelectedTarget { target_key: target }
                }
            }
        }
    }
}

#[component]
fn InlinePopover(target_key: String) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();
    let cited_by = state
        .archive
        .as_ref()
        .map(|a| a.cited_by(&target_key))
        .unwrap_or_default();

    rsx! {
        span { class: "inline-popover",
            onclick: move |e| e.stop_propagation(),
            div { class: "popover-arrow" }
            div { class: "popover-body",
                strong { "{target_key}" }
                span { class: "popover-count",
                    " ({cited_by.len()} refs)"
                }
                if !cited_by.is_empty() {
                    ul { class: "popover-mini-list",
                        for (chunk, _) in cited_by.iter().take(5) {
                            li {
                                Link {
                                    to: Route::DocumentPage { id: chunk.chunk_id.clone() },
                                    "{chunk.title}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SelectedTarget(target_key: String) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();
    let cited_by = state
        .archive
        .as_ref()
        .map(|a| a.cited_by(&target_key))
        .unwrap_or_default();

    let label = target_key
        .split_once(':')
        .map(|(_, v)| v)
        .unwrap_or(&target_key)
        .to_string();
    let count = cited_by.len();

    rsx! {
        div { class: "selected-target",
            h5 { "Selected: {label}" }
            p { class: "selected-count", "{count} references across corpus" }
            if !cited_by.is_empty() {
                ul { class: "selected-list",
                    for (chunk, _citation) in cited_by.iter().take(10) {
                        li {
                            Link {
                                to: Route::DocumentPage { id: chunk.chunk_id.clone() },
                                class: "selected-link",
                                "{chunk.title}"
                            }
                            if !chunk.author.is_empty() {
                                span { class: "selected-author", " — {chunk.author}" }
                            }
                        }
                    }
                }
                if count > 10 {
                    p { class: "selected-more", "...and {count - 10} more" }
                }
            }
        }
    }
}
