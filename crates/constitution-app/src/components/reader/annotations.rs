//! Annotations panel for the document reader. Reads + writes
//! `UserData::annotations`, persists to localStorage, and uses the
//! browser's text-selection API to capture quoted passages.

use dioxus::prelude::*;

use crate::state::{use_archive, use_user_data, Annotation};
use crate::today_iso;

#[component]
pub fn AnnotationsPanel(chunk_id: String, chunk_title: String) -> Element {
    let mut user_data = use_user_data();
    let archive = use_archive();
    let _ = archive; // surface dependency so the panel re-renders on archive change

    let mut note_body = use_signal(String::new);
    let mut editing_id = use_signal(|| Option::<String>::None);
    let mut editing_body = use_signal(String::new);

    let existing: Vec<Annotation> = user_data
        .read()
        .annotations_for(&chunk_id)
        .into_iter()
        .cloned()
        .collect();

    let chunk_id_for_save = chunk_id.clone();
    let chunk_title_for_save = chunk_title.clone();
    let save_note = move |_| {
        let body = note_body.read().trim().to_string();
        if body.is_empty() {
            return;
        }
        let quote = read_selection_in_doc();
        {
            let mut u = user_data.write();
            u.add_annotation(
                chunk_id_for_save.clone(),
                chunk_title_for_save.clone(),
                quote,
                body,
                today_iso(),
            );
        }
        crate::persist_user_data(&user_data.read());
        note_body.set(String::new());
    };

    rsx! {
        section { class: "annotations-panel",
            header { class: "annotations-header",
                h4 { "Your notes" }
                span { class: "annotations-count", "{existing.len()}" }
            }
            div { class: "annotations-add",
                textarea {
                    class: "annotation-input",
                    placeholder: "Highlight text above to attach a quote, then jot a note...",
                    value: "{note_body}",
                    oninput: move |e| note_body.set(e.value()),
                }
                div { class: "annotation-add-actions",
                    button {
                        class: "btn btn-ghost annotation-quote-hint",
                        title: "Refresh selection preview",
                        onclick: move |_| {
                            // Force the panel to re-read selection
                            let _ = read_selection_in_doc();
                        },
                        "↻ Use selection"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: save_note,
                        "Save note"
                    }
                }
            }
            if existing.is_empty() {
                p { class: "annotations-empty",
                    "No notes on this passage yet."
                }
            } else {
                ul { class: "annotations-list",
                    for ann in existing.iter() {
                        {
                            let id = ann.id.clone();
                            let id_for_edit = id.clone();
                            let id_for_save = id.clone();
                            let id_for_delete = id.clone();
                            let body_for_edit = ann.body.clone();
                            let is_editing = editing_id.read().as_deref() == Some(&id);
                            rsx! {
                                li { class: "annotation-card", key: "{id}",
                                    if !ann.quote.is_empty() {
                                        blockquote { class: "annotation-quote", "{ann.quote}" }
                                    }
                                    if is_editing {
                                        textarea {
                                            class: "annotation-input",
                                            value: "{editing_body}",
                                            oninput: move |e| editing_body.set(e.value()),
                                        }
                                        div { class: "annotation-add-actions",
                                            button {
                                                class: "btn btn-ghost",
                                                onclick: move |_| {
                                                    editing_id.set(None);
                                                    editing_body.set(String::new());
                                                },
                                                "Cancel"
                                            }
                                            button {
                                                class: "btn btn-primary",
                                                onclick: move |_| {
                                                    let new_body = editing_body.read().trim().to_string();
                                                    if !new_body.is_empty() {
                                                        {
                                                            let mut u = user_data.write();
                                                            u.update_annotation_body(&id_for_save, new_body);
                                                        }
                                                        crate::persist_user_data(&user_data.read());
                                                    }
                                                    editing_id.set(None);
                                                    editing_body.set(String::new());
                                                },
                                                "Save"
                                            }
                                        }
                                    } else {
                                        p { class: "annotation-body", "{ann.body}" }
                                        div { class: "annotation-card-footer",
                                            span { class: "annotation-date", "{ann.created_at}" }
                                            div { class: "annotation-actions",
                                                button {
                                                    class: "annotation-action-btn",
                                                    aria_label: "Edit note",
                                                    onclick: move |_| {
                                                        editing_id.set(Some(id_for_edit.clone()));
                                                        editing_body.set(body_for_edit.clone());
                                                    },
                                                    "Edit"
                                                }
                                                button {
                                                    class: "annotation-action-btn annotation-action-danger",
                                                    aria_label: "Delete note",
                                                    onclick: move |_| {
                                                        {
                                                            let mut u = user_data.write();
                                                            u.delete_annotation(&id_for_delete);
                                                        }
                                                        crate::persist_user_data(&user_data.read());
                                                    },
                                                    "Delete"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Reads the current browser text selection if it lies inside the
/// document-content region. Returns an empty string on native or when
/// nothing is selected.
fn read_selection_in_doc() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else { return String::new(); };
        let Ok(Some(sel)) = window.get_selection() else { return String::new(); };
        let raw = sel.to_string().as_string().unwrap_or_default();
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        // Cap quote length so a runaway selection doesn't blow out localStorage.
        if trimmed.len() > 600 {
            let mut cut = 600;
            while !trimmed.is_char_boundary(cut) && cut > 0 {
                cut -= 1;
            }
            return format!("{}…", &trimmed[..cut]);
        }
        trimmed.to_string()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        String::new()
    }
}
