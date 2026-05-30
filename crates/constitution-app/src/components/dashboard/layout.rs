use constitution_archive::SearchOptions;
use dioxus::prelude::*;

use crate::components::dashboard::coordinated::CoordinatedDashboard;
use crate::components::shared::{LoadingSpinner, StatTile};
use crate::router::Route;
use crate::state::{use_archive, use_search_state, use_user_data};

#[component]
pub fn DashboardPage() -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading constitutional archive...".to_string() } };
    }

    if let Some(ref err) = state.error {
        return rsx! {
            div { class: "page dashboard-page",
                div { class: "error-banner",
                    h3 { "Archive not loaded" }
                    p { "{err}" }
                    p { "Build the archive with: cargo run --bin build-archive -- --web" }
                }
            }
        };
    }

    let stats = state.stats();

    let top_citations = state
        .archive
        .as_ref()
        .map(|a| a.top_citation_targets(10))
        .unwrap_or_default();

    rsx! {
        div { class: "page dashboard-page",
            header { class: "page-header",
                h2 { "Constitution Research Workbench" }
                p { class: "page-subtitle",
                    "Search, visualize, and compare constitutional texts from around the world."
                }
            }

            if let Some(ref stats) = stats {
                section { class: "stats-grid",
                    StatTile { label: "Chunks".to_string(), value: format_number(stats.chunks) }
                    StatTile { label: "Documents".to_string(), value: format_number(stats.documents) }
                    StatTile { label: "Authors".to_string(), value: format_number(stats.authors) }
                    StatTile { label: "Collections".to_string(), value: stats.collections.to_string() }
                    StatTile { label: "Citations".to_string(), value: format_number(stats.citations) }
                    StatTile { label: "Timeline Events".to_string(), value: stats.events.to_string() }
                    StatTile { label: "Index Terms".to_string(), value: format_number(stats.terms) }
                    StatTile { label: "World Constitutions".to_string(), value: state.world_meta.len().to_string() }
                }
            }

            // Coordinated multi-view dashboard
            section { class: "coord-section",
                CoordinatedDashboard {}
            }

            section { class: "dashboard-grid",
                // Quick search
                div { class: "dashboard-card dashboard-card-wide",
                    h3 { class: "card-title", "Quick Search" }
                    QuickSearch {}
                }

                // Bookmarks & history
                div { class: "dashboard-card",
                    h3 { class: "card-title", "Your library" }
                    UserLibraryPanel {}
                }

                // Top citations
                div { class: "dashboard-card",
                    h3 { class: "card-title", "Most Referenced" }
                    if top_citations.is_empty() {
                        p { class: "card-empty", "No citations found." }
                    } else {
                        ol { class: "top-citations-list",
                            for (key, count) in top_citations.iter() {
                                li { class: "citation-item",
                                    span { class: "citation-key", "{key}" }
                                    span { class: "citation-count", "{count}" }
                                }
                            }
                        }
                    }
                    Link { to: crate::router::Route::GraphPage {},
                        class: "card-action",
                        "View Full Graph"
                    }
                }

                // Region overview
                div { class: "dashboard-card",
                    h3 { class: "card-title", "World Constitutions by Region" }
                    {
                        let mut region_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
                        for m in &state.world_meta {
                            *region_counts.entry(&m.region).or_insert(0) += 1;
                        }
                        let mut regions: Vec<_> = region_counts.into_iter().collect();
                        regions.sort_by(|a, b| b.1.cmp(&a.1));
                        rsx! {
                            div { class: "region-bars",
                                for (region, count) in regions.iter() {
                                    div { class: "region-bar-row",
                                        span { class: "region-bar-label", "{region}" }
                                        div { class: "region-bar-track",
                                            div {
                                                class: "region-bar-fill",
                                                style: "width: {(*count as f64 / state.world_meta.len().max(1) as f64 * 100.0):.0}%",
                                            }
                                        }
                                        span { class: "region-bar-count", "{count}" }
                                    }
                                }
                            }
                        }
                    }
                    Link { to: crate::router::Route::WorldPage {},
                        class: "card-action",
                        "Explore Map"
                    }
                }
            }

            // Navigation panels
            section { class: "dashboard-panels",
                div { class: "panel",
                    h3 { class: "panel-title", "Full Search" }
                    p { class: "panel-desc", "Full-text BM25 search with fuzzy matching, filters, and grouped results." }
                    Link { to: crate::router::Route::SearchPage {},
                        class: "panel-action",
                        "Open Search"
                    }
                }
                div { class: "panel",
                    h3 { class: "panel-title", "Citation Network" }
                    p { class: "panel-desc", "Force-directed graph of how founders, clauses, and essays reference each other." }
                    Link { to: crate::router::Route::GraphPage {},
                        class: "panel-action",
                        "Explore Graph"
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
                div { class: "panel",
                    h3 { class: "panel-title", "Blog & Analysis" }
                    p { class: "panel-desc", "Commentary, analysis, and interactive explorations with embedded widgets." }
                    Link { to: crate::router::Route::BlogIndexPage {},
                        class: "panel-action",
                        "Read Blog"
                    }
                }
            }
        }
    }
}

#[component]
fn QuickSearch() -> Element {
    let archive_state = use_archive();
    let mut search_state = use_search_state();
    let mut query = use_signal(String::new);
    let mut quick_results = use_signal(Vec::<constitution_archive::SearchHit>::new);

    rsx! {
        div { class: "quick-search",
            div { class: "search-bar",
                input {
                    class: "search-input",
                    r#type: "text",
                    placeholder: "Try: \"due process\", \"federalism\", \"executive power\"...",
                    value: "{query}",
                    oninput: move |e| {
                        let val = e.value();
                        query.set(val.clone());
                        if val.len() >= 3 {
                            let state = archive_state.read();
                            let opts = SearchOptions {
                                limit: 5,
                                fuzzy_distance: 1,
                                snippet_window: 120,
                                ..Default::default()
                            };
                            let results = state.search(
                                &val,
                                &constitution_archive::Filter::default(),
                                &opts,
                            );
                            quick_results.set(results);
                        } else {
                            quick_results.set(Vec::new());
                        }
                    },
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            let q = query.read().clone();
                            if !q.is_empty() {
                                let mut ss = search_state.write();
                                ss.query = q;
                                ss.results = quick_results.read().clone();
                                ss.total_results = quick_results.read().len();
                            }
                        }
                    },
                }
                Link {
                    to: crate::router::Route::SearchPage {},
                    class: "search-button",
                    "Full Search"
                }
            }
            if !quick_results.read().is_empty() {
                div { class: "quick-results",
                    for hit in quick_results.read().iter() {
                        {
                            let chunk = archive_state.read().chunk(&hit.chunk_id);
                            rsx! {
                                Link {
                                    to: crate::router::Route::DocumentPage { id: hit.chunk_id.clone() },
                                    class: "quick-result",
                                    div { class: "quick-result-title",
                                        {chunk.as_ref().map(|c| c.title.as_str()).unwrap_or("Unknown")}
                                    }
                                    if !hit.snippet.text.is_empty() {
                                        div {
                                            class: "quick-result-snippet",
                                            dangerous_inner_html: "{hit.snippet.text}",
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

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

#[component]
fn UserLibraryPanel() -> Element {
    let user_data = use_user_data();
    let mut tab = use_signal(|| LibraryTab::History);
    let data = user_data.read();
    let history = data.history.clone();
    let bookmarks = data.bookmarks.clone();
    let recent = data.recent_searches.clone();
    drop(data);

    let current = *tab.read();
    let counts = (
        history.len(),
        bookmarks.len(),
        recent.len(),
    );

    rsx! {
        div { class: "library-panel",
            div { class: "library-tabs", role: "tablist",
                button {
                    role: "tab",
                    aria_selected: if current == LibraryTab::History { "true" } else { "false" },
                    class: if current == LibraryTab::History { "library-tab library-tab-active" } else { "library-tab" },
                    onclick: move |_| tab.set(LibraryTab::History),
                    "Recent ({counts.0})"
                }
                button {
                    role: "tab",
                    aria_selected: if current == LibraryTab::Bookmarks { "true" } else { "false" },
                    class: if current == LibraryTab::Bookmarks { "library-tab library-tab-active" } else { "library-tab" },
                    onclick: move |_| tab.set(LibraryTab::Bookmarks),
                    "Bookmarks ({counts.1})"
                }
                button {
                    role: "tab",
                    aria_selected: if current == LibraryTab::Searches { "true" } else { "false" },
                    class: if current == LibraryTab::Searches { "library-tab library-tab-active" } else { "library-tab" },
                    onclick: move |_| tab.set(LibraryTab::Searches),
                    "Searches ({counts.2})"
                }
            }
            div { class: "library-body",
                {
                    match current {
                        LibraryTab::History => rsx! {
                            if history.is_empty() {
                                p { class: "card-empty",
                                    "No history yet. Open a document and it'll show up here."
                                }
                            }
                            for h in history.iter().take(8) {
                                Link {
                                    to: Route::DocumentPage { id: h.chunk_id.clone() },
                                    class: "library-item",
                                    div { class: "library-item-title", "{h.title}" }
                                    div { class: "library-item-meta", "{h.collection}" }
                                }
                            }
                        },
                        LibraryTab::Bookmarks => rsx! {
                            if bookmarks.is_empty() {
                                p { class: "card-empty",
                                    "No bookmarks yet. Star a document with the ☆ button."
                                }
                            }
                            for b in bookmarks.iter().take(8) {
                                Link {
                                    to: Route::DocumentPage { id: b.chunk_id.clone() },
                                    class: "library-item",
                                    div { class: "library-item-title", "★ {b.title}" }
                                    div { class: "library-item-meta", "{b.collection}" }
                                }
                            }
                        },
                        LibraryTab::Searches => rsx! {
                            if recent.is_empty() {
                                p { class: "card-empty",
                                    "Run a search to see your recent queries here."
                                }
                            }
                            for q in recent.iter().take(8) {
                                Link {
                                    to: Route::SearchPage {},
                                    class: "library-item",
                                    div { class: "library-item-title", "{q}" }
                                    div { class: "library-item-meta", "Recent search" }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LibraryTab {
    History,
    Bookmarks,
    Searches,
}
