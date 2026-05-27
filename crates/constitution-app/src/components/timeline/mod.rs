mod timeline_view;

use dioxus::prelude::*;

use crate::components::shared::LoadingSpinner;
use crate::state::{use_archive, use_selection};
use timeline_view::TimelineSvg;

#[component]
pub fn TimelinePage() -> Element {
    let archive_state = use_archive();
    let selection = use_selection();

    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading archive...".to_string() } };
    }

    let phases = state
        .archive
        .as_ref()
        .map(|a| a.timeline_by_phase())
        .unwrap_or_default();

    let selected_key = selection.read().target_key();

    rsx! {
        div { class: "page timeline-page",
            header { class: "page-header",
                h2 { "Constitutional Process Timeline" }
                p { class: "page-subtitle",
                    "Follow the drafting and ratification of the U.S. Constitution."
                }
            }
            TimelineSvg { selected_key: selected_key }
            section { class: "timeline-phases",
                for (phase_label, events) in phases.iter() {
                    div { class: "timeline-phase",
                        h3 { class: "phase-label", "{phase_label}" }
                        div { class: "phase-events",
                            for event in events.iter() {
                                div { class: "event-card",
                                    div { class: "event-header",
                                        span { class: "event-date", "{event.date}" }
                                        strong { class: "event-title", "{event.title}" }
                                    }
                                    p { class: "event-summary", "{event.summary}" }
                                    if !event.source_chunks.is_empty() {
                                        div { class: "event-sources",
                                            span { class: "source-count",
                                                "{event.source_chunks.len()} source chunks"
                                            }
                                            for chunk_id in event.source_chunks.iter().take(3) {
                                                Link {
                                                    to: crate::router::Route::DocumentPage { id: chunk_id.clone() },
                                                    class: "source-link",
                                                    "{chunk_id}"
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
