use dioxus::prelude::*;

use crate::components::shared::{LoadingSpinner, StatTile};
use crate::state::use_archive;

#[component]
pub fn DashboardPage() -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading constitutional archive...".to_string() } };
    }

    if let Some(ref err) = state.error {
        return rsx! {
            div { class: "error-banner",
                h3 { "Failed to load archive" }
                p { "{err}" }
            }
        };
    }

    let stats = state.stats();

    rsx! {
        div { class: "page dashboard-page",
            header { class: "page-header",
                h2 { "Constitution Research Workbench" }
                p { class: "page-subtitle",
                    "Search, visualize, and compare constitutional texts from around the world."
                }
            }
            if let Some(stats) = stats {
                section { class: "stats-grid",
                    StatTile { label: "Chunks".to_string(), value: stats.chunks.to_string() }
                    StatTile { label: "Documents".to_string(), value: stats.documents.to_string() }
                    StatTile { label: "Authors".to_string(), value: stats.authors.to_string() }
                    StatTile { label: "Collections".to_string(), value: stats.collections.to_string() }
                    StatTile { label: "Citations".to_string(), value: stats.citations.to_string() }
                    StatTile { label: "Timeline Events".to_string(), value: stats.events.to_string() }
                    StatTile { label: "Index Terms".to_string(), value: stats.terms.to_string() }
                    StatTile { label: "World Constitutions".to_string(), value: state.world_meta.len().to_string() }
                }
            }
            section { class: "dashboard-panels",
                div { class: "panel",
                    h3 { class: "panel-title", "Quick Search" }
                    p { class: "panel-desc", "Full-text BM25 search with fuzzy matching across the entire corpus." }
                    Link { to: crate::router::Route::SearchPage {},
                        class: "panel-action",
                        "Open Search"
                    }
                }
                div { class: "panel",
                    h3 { class: "panel-title", "Citation Network" }
                    p { class: "panel-desc", "Explore how founders, clauses, and essays reference each other." }
                    Link { to: crate::router::Route::GraphPage {},
                        class: "panel-action",
                        "Explore Graph"
                    }
                }
                div { class: "panel",
                    h3 { class: "panel-title", "World Constitutions" }
                    p { class: "panel-desc", "Compare provisions across 194 national constitutions." }
                    Link { to: crate::router::Route::WorldPage {},
                        class: "panel-action",
                        "View Map"
                    }
                }
                div { class: "panel",
                    h3 { class: "panel-title", "Process Timeline" }
                    p { class: "panel-desc", "Follow the constitutional drafting and ratification journey." }
                    Link { to: crate::router::Route::TimelinePage {},
                        class: "panel-action",
                        "View Timeline"
                    }
                }
            }
        }
    }
}
