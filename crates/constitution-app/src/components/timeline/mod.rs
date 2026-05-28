mod timeline_view;

use dioxus::prelude::*;

use crate::components::shared::LoadingSpinner;
use crate::router::Route;
use crate::state::{use_archive, use_selection};
use timeline_view::TimelineSvg;

#[component]
pub fn TimelinePage() -> Element {
    let archive_state = use_archive();
    let selection = use_selection();
    let mut phase_filter = use_signal(|| Option::<String>::None);
    let mut expanded: Signal<Option<String>> = use_signal(|| None);

    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading archive...".to_string() } };
    }

    let archive = match state.archive.as_ref() {
        Some(a) => a,
        None => return rsx! {
            div { class: "page timeline-page",
                p { "Archive not loaded." }
            }
        },
    };

    let phases = archive.timeline_by_phase();

    let sel = selection.read();
    let selected_key = sel.target_key();

    // Build the set of events related to the selection
    let related_event_ids: std::collections::HashSet<String> = if let Some(ref key) = selected_key {
        let cited = archive.cited_by(key);
        let mut event_ids = std::collections::HashSet::new();
        for (chunk, _) in &cited {
            for ev in archive.events_for_chunk(&chunk.chunk_id) {
                event_ids.insert(ev.id.clone());
            }
        }
        event_ids
    } else {
        std::collections::HashSet::new()
    };

    let active_phase = phase_filter.read().clone();

    rsx! {
        div { class: "page timeline-page",
            header { class: "page-header",
                h2 { "Constitutional Process Timeline" }
                p { class: "page-subtitle",
                    "Follow the drafting and ratification of the U.S. Constitution from the failed Articles of Confederation through the Bill of Rights."
                }
                if selected_key.is_some() && !related_event_ids.is_empty() {
                    div { class: "page-callout",
                        "{related_event_ids.len()} events highlighted (related to current selection)"
                    }
                }
            }
            TimelineSvg { selected_key: selected_key.clone() }

            div { class: "phase-filter-bar",
                button {
                    class: if active_phase.is_none() { "phase-pill phase-pill-active" } else { "phase-pill" },
                    onclick: move |_| phase_filter.set(None),
                    "All phases"
                }
                for (label, _events) in phases.iter() {
                    {
                        let l = label.to_string();
                        let is_active = active_phase.as_deref() == Some(label);
                        rsx! {
                            button {
                                class: if is_active { "phase-pill phase-pill-active" } else { "phase-pill" },
                                onclick: move |_| phase_filter.set(Some(l.clone())),
                                "{label}"
                            }
                        }
                    }
                }
            }

            section { class: "timeline-phases",
                for (phase_label, events) in phases.iter() {
                    {
                        let phase_label_str = phase_label.to_string();
                        let visible = active_phase
                            .as_ref()
                            .map(|p| p == &phase_label_str)
                            .unwrap_or(true);
                        if !visible {
                            return rsx! {};
                        }
                        rsx! {
                            div { class: "timeline-phase",
                                h3 { class: "phase-label", "{phase_label}" }
                                div { class: "phase-events",
                                    for event in events.iter() {
                                        {
                                            let event_id = event.id.clone();
                                            let event_id_for_use = event_id.clone();
                                            let event_id_for_close = event_id.clone();
                                            let is_expanded = expanded.read().as_deref() == Some(&event_id);
                                            let is_related = related_event_ids.contains(&event_id);
                                            let card_class = if is_related {
                                                "event-card event-card-highlighted"
                                            } else if related_event_ids.is_empty() {
                                                "event-card"
                                            } else {
                                                "event-card event-card-dimmed"
                                            };
                                            rsx! {
                                                div {
                                                    class: "{card_class}",
                                                    key: "{event.id}",
                                                    button {
                                                        class: "event-card-toggle",
                                                        onclick: move |_| {
                                                            let cur = expanded.read().clone();
                                                            if cur.as_deref() == Some(&event_id_for_use) {
                                                                expanded.set(None);
                                                            } else {
                                                                expanded.set(Some(event_id_for_use.clone()));
                                                            }
                                                        },
                                                        div { class: "event-header",
                                                            span { class: "event-date", "{event.date}" }
                                                            strong { class: "event-title", "{event.title}" }
                                                            span { class: "event-toggle-icon",
                                                                if is_expanded { "v" } else { ">" }
                                                            }
                                                        }
                                                    }
                                                    if is_expanded {
                                                        div { class: "event-detail",
                                                            p { class: "event-summary", "{event.summary}" }
                                                            if !event.actors.is_empty() {
                                                                div { class: "event-detail-row",
                                                                    span { class: "event-detail-label", "Actors:" }
                                                                    for actor in event.actors.iter() {
                                                                        span { class: "event-actor", "{actor}" }
                                                                    }
                                                                }
                                                            }
                                                            if !event.locations.is_empty() {
                                                                div { class: "event-detail-row",
                                                                    span { class: "event-detail-label", "Where:" }
                                                                    for loc in event.locations.iter() {
                                                                        span { class: "event-location", "{loc}" }
                                                                    }
                                                                }
                                                            }
                                                            if !event.source_chunks.is_empty() {
                                                                div { class: "event-detail-row",
                                                                    span { class: "event-detail-label", "Sources:" }
                                                                    div { class: "event-sources-list",
                                                                        for chunk_id in event.source_chunks.iter() {
                                                                            Link {
                                                                                to: Route::DocumentPage { id: chunk_id.clone() },
                                                                                class: "event-source-link",
                                                                                "{chunk_id}"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            if !event.cross_refs.is_empty() {
                                                                div { class: "event-detail-row",
                                                                    span { class: "event-detail-label", "Related:" }
                                                                    for xref in event.cross_refs.iter() {
                                                                        span { class: "event-xref", "{xref}" }
                                                                    }
                                                                }
                                                            }
                                                            button {
                                                                class: "event-close",
                                                                onclick: move |_| expanded.set(None),
                                                                if event_id_for_close.is_empty() { "Close" } else { "Close" }
                                                            }
                                                        }
                                                    } else {
                                                        p { class: "event-summary", "{event.summary}" }
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
}
