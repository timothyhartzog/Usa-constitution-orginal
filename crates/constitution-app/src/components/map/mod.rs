mod world_map;

use dioxus::prelude::*;

use crate::components::shared::LoadingSpinner;
use crate::state::{use_archive, use_selection, SelectionKind, SelectionState};
use world_map::WorldMapSvg;

#[component]
pub fn WorldPage() -> Element {
    let archive_state = use_archive();
    let mut selection = use_selection();

    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading archive...".to_string() } };
    }

    let world_meta = &state.world_meta;
    let selected_country = match &selection.read().kind {
        SelectionKind::Country(c) => Some(c.clone()),
        _ => None,
    };

    let regions: Vec<&str> = {
        let mut r: Vec<&str> = world_meta.iter().map(|m| m.region.as_str()).collect();
        r.sort_unstable();
        r.dedup();
        r
    };

    let mut region_filter = use_signal(|| Option::<String>::None);

    let filtered_meta: Vec<_> = world_meta
        .iter()
        .filter(|m| {
            region_filter
                .read()
                .as_ref()
                .map(|r| m.region == *r)
                .unwrap_or(true)
        })
        .collect();

    rsx! {
        div { class: "page world-page",
            header { class: "page-header",
                h2 { "World Constitutions" }
                p { class: "page-subtitle",
                    "Explore and compare {world_meta.len()} national constitutions."
                }
            }
            div { class: "world-layout",
                aside { class: "world-sidebar",
                    div { class: "region-filter",
                        h4 { "Filter by Region" }
                        button {
                            class: if region_filter.read().is_none() { "region-btn region-btn-active" } else { "region-btn" },
                            onclick: move |_| region_filter.set(None),
                            "All ({world_meta.len()})"
                        }
                        for region in regions.iter() {
                            {
                                let count = world_meta.iter().filter(|m| m.region == *region).count();
                                let r = region.to_string();
                                let active = region_filter.read().as_deref() == Some(*region);
                                rsx! {
                                    button {
                                        class: if active { "region-btn region-btn-active" } else { "region-btn" },
                                        onclick: move |_| region_filter.set(Some(r.clone())),
                                        "{region} ({count})"
                                    }
                                }
                            }
                        }
                    }
                    div { class: "country-list",
                        h4 { "{filtered_meta.len()} Constitutions" }
                        for meta in filtered_meta.iter() {
                            button {
                                class: if selected_country.as_deref() == Some(&meta.country_id) {
                                    "country-item country-item-active"
                                } else {
                                    "country-item"
                                },
                                onclick: {
                                    let id = meta.country_id.clone();
                                    move |_| selection.set(SelectionState::select_country(id.clone()))
                                },
                                div { class: "country-name", "{meta.country}" }
                                div { class: "country-region", "{meta.region}" }
                            }
                        }
                    }
                }
                main { class: "world-main",
                    WorldMapSvg { selected_country: selected_country.clone() }
                    if let Some(ref country_id) = selected_country {
                        { CountryDetail(country_id, world_meta) }
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn CountryDetail(country_id: &str, meta: &[crate::state::WorldConstitutionMeta]) -> Element {
    let entry = meta.iter().find(|m| m.country_id == country_id);

    match entry {
        Some(m) => rsx! {
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
                        span { class: "detail-label", "Document ID" }
                        span { class: "detail-value", "{m.document_id}" }
                    }
                }
                Link {
                    to: crate::router::Route::SearchPage {},
                    class: "detail-action",
                    "Search this constitution"
                }
            }
        },
        None => rsx! {
            p { "No metadata for {country_id}." }
        },
    }
}
