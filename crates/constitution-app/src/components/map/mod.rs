mod world_map;

use constitution_archive::{Filter, FilterValue, SearchOptions};
use dioxus::prelude::*;

use crate::components::shared::LoadingSpinner;
use crate::router::Route;
use crate::state::{use_archive, use_selection, SelectionKind, SelectionState, WorldConstitutionMeta};
use world_map::WorldMapSvg;

#[component]
pub fn WorldPage() -> Element {
    let archive_state = use_archive();
    let mut selection = use_selection();
    let mut region_filter: Signal<Option<String>> = use_signal(|| None);
    let mut search_term = use_signal(String::new);
    let mut compare_topic = use_signal(String::new);

    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading archive...".to_string() } };
    }

    let world_meta = &state.world_meta;
    let selected_country = match &selection.read().kind {
        SelectionKind::Country(c) => Some(c.clone()),
        _ => None,
    };

    let regions: Vec<String> = {
        let mut r: Vec<String> = world_meta.iter().map(|m| m.region.clone()).collect();
        r.sort();
        r.dedup();
        r
    };

    let term = search_term.read().to_lowercase();
    let region = region_filter.read().clone();

    let filtered_meta: Vec<&WorldConstitutionMeta> = world_meta
        .iter()
        .filter(|m| {
            let region_ok = region
                .as_ref()
                .map(|r| &m.region == r)
                .unwrap_or(true);
            let term_ok = term.is_empty()
                || m.country.to_lowercase().contains(&term)
                || m.country_id.to_lowercase().contains(&term);
            region_ok && term_ok
        })
        .collect();

    rsx! {
        div { class: "page world-page",
            header { class: "page-header",
                h2 { "World Constitutions" }
                p { class: "page-subtitle",
                    "Explore and compare {world_meta.len()} national constitutions from across the globe."
                }
            }
            div { class: "world-layout",
                aside { class: "world-sidebar",
                    div { class: "world-search",
                        input {
                            class: "world-search-input",
                            r#type: "text",
                            placeholder: "Find country...",
                            value: "{search_term}",
                            oninput: move |e| search_term.set(e.value()),
                        }
                    }
                    div { class: "region-filter",
                        h4 { "Filter by Region" }
                        button {
                            class: if region_filter.read().is_none() { "region-btn region-btn-active" } else { "region-btn" },
                            onclick: move |_| region_filter.set(None),
                            "All ({world_meta.len()})"
                        }
                        for r in regions.iter() {
                            {
                                let count = world_meta.iter().filter(|m| m.region == *r).count();
                                let r_str = r.clone();
                                let active = region_filter.read().as_deref() == Some(r);
                                rsx! {
                                    button {
                                        class: if active { "region-btn region-btn-active" } else { "region-btn" },
                                        onclick: move |_| region_filter.set(Some(r_str.clone())),
                                        "{r} ({count})"
                                    }
                                }
                            }
                        }
                    }
                    div { class: "country-list",
                        h4 { "{filtered_meta.len()} Constitutions" }
                        for meta in filtered_meta.iter() {
                            {
                                let id = meta.country_id.clone();
                                let active = selected_country.as_deref() == Some(&meta.country_id);
                                rsx! {
                                    button {
                                        class: if active {
                                            "country-item country-item-active"
                                        } else {
                                            "country-item"
                                        },
                                        onclick: move |_| selection.set(SelectionState::select_country(id.clone())),
                                        div { class: "country-name", "{meta.country}" }
                                        div { class: "country-region", "{meta.region}" }
                                    }
                                }
                            }
                        }
                    }
                }
                main { class: "world-main",
                    WorldMapSvg { selected_country: selected_country.clone() }
                    if let Some(ref country_id) = selected_country {
                        CountryDetail { country_id: country_id.clone() }
                        ComparePanel {
                            country_id: country_id.clone(),
                            topic: compare_topic.read().clone(),
                            on_topic_change: move |t: String| compare_topic.set(t),
                        }
                    } else {
                        div { class: "world-empty",
                            h3 { "Select a country" }
                            p { "Click a country in the sidebar or a region bubble on the map to see its constitution." }
                            div { class: "region-summary",
                                h4 { "Constitutions by Region" }
                                {
                                    let mut by_region: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
                                    for m in world_meta {
                                        *by_region.entry(m.region.as_str()).or_insert(0) += 1;
                                    }
                                    let mut entries: Vec<_> = by_region.into_iter().collect();
                                    entries.sort_by(|a, b| b.1.cmp(&a.1));
                                    rsx! {
                                        ul { class: "region-summary-list",
                                            for (region, count) in entries.iter() {
                                                li {
                                                    span { class: "region-summary-region", "{region}" }
                                                    span { class: "region-summary-count", "{count} constitutions" }
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

#[component]
fn CountryDetail(country_id: String) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();
    let world_meta = &state.world_meta;

    let entry = world_meta.iter().find(|m| m.country_id == country_id);

    match entry {
        Some(m) => {
            let archive = state.archive.as_ref();
            let chunks_in_country = archive
                .map(|a| {
                    a.chunks()
                        .iter()
                        .filter(|c| c.document_id.contains(&country_id.to_lowercase()))
                        .count()
                })
                .unwrap_or(0);

            rsx! {
                div { class: "country-detail",
                    h3 { "{m.country}" }
                    div { class: "detail-grid",
                        div { class: "detail-item",
                            span { class: "detail-label", "Region" }
                            span { class: "detail-value", "{m.region}" }
                        }
                        div { class: "detail-item",
                            span { class: "detail-label", "Word Count" }
                            span { class: "detail-value", "{m.word_count}" }
                        }
                        div { class: "detail-item",
                            span { class: "detail-label", "Chunks Indexed" }
                            span { class: "detail-value", "{chunks_in_country}" }
                        }
                        div { class: "detail-item",
                            span { class: "detail-label", "Document ID" }
                            span { class: "detail-value", "{m.document_id}" }
                        }
                    }
                    Link {
                        to: Route::SearchPage {},
                        class: "detail-action",
                        "Search the corpus"
                    }
                }
            }
        }
        None => rsx! {
            div { class: "country-detail",
                p { "No metadata for {country_id}." }
            }
        },
    }
}

#[component]
fn ComparePanel(country_id: String, topic: String, on_topic_change: EventHandler<String>) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();
    let mut current_topic = use_signal(|| topic.clone());

    let archive = match state.archive.as_ref() {
        Some(a) => a,
        None => return rsx! { div {} },
    };

    let topic_val = current_topic.read().clone();

    let country_results = if topic_val.is_empty() {
        Vec::new()
    } else {
        // Search within this country's document
        let term_lower = country_id.to_lowercase();
        archive
            .chunks()
            .iter()
            .filter(|c| c.document_id.to_lowercase().contains(&term_lower))
            .filter(|c| c.text.to_lowercase().contains(&topic_val.to_lowercase()))
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
    };

    let us_results = if topic_val.is_empty() {
        Vec::new()
    } else {
        let filter = Filter::default().with(FilterValue::Collection(vec![
            "constitution".to_string(),
            "bill_of_rights".to_string(),
        ]));
        let opts = SearchOptions {
            limit: 3,
            snippet_window: 140,
            ..Default::default()
        };
        archive.search(&topic_val, &filter, &opts)
    };

    rsx! {
        div { class: "compare-panel",
            h3 { "Compare \"{country_id}\" to the U.S. Constitution" }
            div { class: "compare-input",
                input {
                    r#type: "text",
                    class: "compare-input-field",
                    placeholder: "Try: \"freedom\", \"president\", \"judiciary\"...",
                    value: "{current_topic}",
                    oninput: move |e| {
                        let v = e.value();
                        current_topic.set(v.clone());
                        on_topic_change.call(v);
                    },
                }
            }
            if !topic_val.is_empty() {
                div { class: "compare-grid",
                    div { class: "compare-col",
                        h4 { class: "compare-col-label", "{country_id} ({country_results.len()} matches)" }
                        if country_results.is_empty() {
                            p { class: "compare-empty", "No matches found." }
                        }
                        for ch in country_results.iter() {
                            div { class: "compare-result",
                                div { class: "compare-result-title", "{ch.chunk_id}" }
                                p { class: "compare-result-preview", "{ch.ensured_preview()}" }
                                Link {
                                    to: Route::DocumentPage { id: ch.chunk_id.clone() },
                                    class: "compare-result-link",
                                    "Read full passage"
                                }
                            }
                        }
                    }
                    div { class: "compare-col",
                        h4 { class: "compare-col-label", "U.S. Constitution & Bill of Rights ({us_results.len()} matches)" }
                        if us_results.is_empty() {
                            p { class: "compare-empty", "No matches found." }
                        }
                        for hit in us_results.iter() {
                            {
                                let chunk = state.chunk(&hit.chunk_id);
                                let title = chunk.as_ref().map(|c| c.title.clone()).unwrap_or_default();
                                rsx! {
                                    div { class: "compare-result",
                                        div { class: "compare-result-title", "{title}" }
                                        if !hit.snippet.text.is_empty() {
                                            p {
                                                class: "compare-result-preview",
                                                dangerous_inner_html: "{hit.snippet.text}",
                                            }
                                        }
                                        Link {
                                            to: Route::DocumentPage { id: hit.chunk_id.clone() },
                                            class: "compare-result-link",
                                            "Read full passage"
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
