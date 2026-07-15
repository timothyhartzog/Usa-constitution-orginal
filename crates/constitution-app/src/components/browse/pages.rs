//! Index + detail pages for browsing the corpus by author and by
//! collection. All pages re-aggregate on render because the underlying
//! `Archive` is immutable for the session; aggregation over 17K chunks
//! takes well under 50ms in release mode.

use dioxus::prelude::*;

use crate::components::browse::aggregate::{
    build_author_dossiers, build_collection_dossiers, find_author, find_collection, AuthorDossier,
    CollectionDossier,
};
use crate::components::shared::{LoadingSpinner, PermalinkButton};
use crate::router::Route;
use crate::state::use_archive;

#[component]
pub fn AuthorsIndexPage() -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading...".to_string() } };
    }
    let Some(archive) = state.archive.as_ref() else {
        return rsx! { p { class: "card-empty", "Archive not loaded." } };
    };

    let dossiers = build_author_dossiers(archive.chunks());
    let max_count = dossiers.first().map(|d| d.chunk_count).unwrap_or(1).max(1);

    rsx! {
        div { class: "page browse-page",
            header { class: "page-header",
                div { class: "page-header-row",
                    div {
                        h2 { "Authors" }
                        p { class: "page-subtitle",
                            "{dossiers.len()} authors across the corpus, sorted by number of chunks."
                        }
                    }
                }
            }
            div { class: "browse-grid",
                for d in dossiers.iter() {
                    AuthorCard { dossier: d.clone(), max_count: max_count }
                }
            }
        }
    }
}

#[component]
fn AuthorCard(dossier: AuthorDossier, max_count: usize) -> Element {
    let pct = ((dossier.chunk_count as f64 / max_count as f64) * 100.0).round() as u32;
    let date_label = dossier
        .date_range
        .as_ref()
        .map(|(a, b)| {
            if a == b {
                a.clone()
            } else {
                format!("{a} – {b}")
            }
        })
        .unwrap_or_default();

    rsx! {
        Link {
            to: Route::AuthorPage { slug: dossier.slug.clone() },
            class: "browse-card",
            div { class: "browse-card-header",
                h3 { class: "browse-card-title", "{dossier.name}" }
                span { class: "browse-card-count", "{dossier.chunk_count}" }
            }
            div { class: "browse-bar-track",
                div {
                    class: "browse-bar-fill",
                    style: "width: {pct}%;",
                }
            }
            div { class: "browse-card-meta",
                if !date_label.is_empty() {
                    span { class: "browse-meta-item", "{date_label}" }
                }
                span { class: "browse-meta-item",
                    "{dossier.document_count} documents"
                }
                for (col, _) in dossier.collections.iter().take(3) {
                    span { class: "browse-meta-chip", "{col}" }
                }
            }
        }
    }
}

#[component]
pub fn AuthorPage(slug: String) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading...".to_string() } };
    }
    let Some(archive) = state.archive.as_ref() else {
        return rsx! { p { class: "card-empty", "Archive not loaded." } };
    };

    let dossiers = build_author_dossiers(archive.chunks());
    let Some(dossier) = find_author(&dossiers, &slug) else {
        return rsx! {
            div { class: "page",
                h2 { "Author not found" }
                p { "No author matches \"{slug}\"." }
                Link { to: Route::AuthorsIndexPage {}, "Back to all authors" }
            }
        };
    };

    let date_label = dossier
        .date_range
        .as_ref()
        .map(|(a, b)| {
            if a == b {
                a.clone()
            } else {
                format!("{a} – {b}")
            }
        })
        .unwrap_or_else(|| "no dates".to_string());

    let samples: Vec<constitution_archive::Chunk> = dossier
        .sample_chunk_ids
        .iter()
        .filter_map(|id| archive.chunk(id).ok().cloned())
        .collect();

    rsx! {
        div { class: "page browse-detail-page",
            header { class: "page-header",
                div { class: "browse-breadcrumb",
                    Link { to: Route::AuthorsIndexPage {}, "All authors" }
                    span { class: "breadcrumb-sep", " / " }
                    span { "{dossier.name}" }
                }
                div { class: "page-header-row",
                    div {
                        h2 { "{dossier.name}" }
                        p { class: "page-subtitle",
                            "{dossier.chunk_count} chunks · {dossier.document_count} documents · {date_label}"
                        }
                    }
                    PermalinkButton { label: Some("Share".to_string()) }
                }
            }

            div { class: "browse-detail-grid",
                section { class: "browse-detail-card",
                    h4 { "Collections" }
                    if dossier.collections.is_empty() {
                        p { class: "card-empty", "No collection metadata." }
                    } else {
                        ul { class: "browse-key-list",
                            for (col, n) in dossier.collections.iter() {
                                li {
                                    Link {
                                        to: Route::CollectionPage { slug: crate::components::browse::slugify(col) },
                                        class: "browse-key-link",
                                        span { class: "browse-key-name", "{col}" }
                                        span { class: "browse-key-count", "{n}" }
                                    }
                                }
                            }
                        }
                    }
                }
                section { class: "browse-detail-card",
                    h4 { "Top issues" }
                    if dossier.top_issues.is_empty() {
                        p { class: "card-empty", "No issue tags recorded." }
                    } else {
                        div { class: "browse-tag-cloud",
                            for (tag, n) in dossier.top_issues.iter() {
                                span { class: "browse-tag",
                                    "{tag}"
                                    span { class: "browse-tag-count", "{n}" }
                                }
                            }
                        }
                    }
                }
                section { class: "browse-detail-card",
                    h4 { "Clauses cited" }
                    if dossier.top_clauses.is_empty() {
                        p { class: "card-empty", "No clause references." }
                    } else {
                        div { class: "browse-tag-cloud",
                            for (clause, n) in dossier.top_clauses.iter() {
                                span { class: "browse-tag",
                                    "{clause}"
                                    span { class: "browse-tag-count", "{n}" }
                                }
                            }
                        }
                    }
                }
            }

            if !samples.is_empty() {
                section { class: "browse-samples",
                    h3 { "Representative passages" }
                    div { class: "siblings-grid",
                        for sample in samples.iter() {
                            Link {
                                to: Route::DocumentPage { id: sample.chunk_id.clone() },
                                class: "sibling-card",
                                div { class: "sibling-id", "{sample.title}" }
                                p { class: "sibling-preview", "{sample.ensured_preview()}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn CollectionsIndexPage() -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading...".to_string() } };
    }
    let Some(archive) = state.archive.as_ref() else {
        return rsx! { p { class: "card-empty", "Archive not loaded." } };
    };

    let dossiers = build_collection_dossiers(archive.chunks());
    let max_count = dossiers.first().map(|d| d.chunk_count).unwrap_or(1).max(1);

    rsx! {
        div { class: "page browse-page",
            header { class: "page-header",
                div { class: "page-header-row",
                    div {
                        h2 { "Collections" }
                        p { class: "page-subtitle",
                            "{dossiers.len()} collections, sorted by number of chunks."
                        }
                    }
                }
            }
            div { class: "browse-grid",
                for d in dossiers.iter() {
                    CollectionCard { dossier: d.clone(), max_count: max_count }
                }
            }
        }
    }
}

#[component]
fn CollectionCard(dossier: CollectionDossier, max_count: usize) -> Element {
    let pct = ((dossier.chunk_count as f64 / max_count as f64) * 100.0).round() as u32;
    let date_label = dossier
        .date_range
        .as_ref()
        .map(|(a, b)| {
            if a == b {
                a.clone()
            } else {
                format!("{a} – {b}")
            }
        })
        .unwrap_or_default();
    let display = dossier.name.replace('_', " ");

    rsx! {
        Link {
            to: Route::CollectionPage { slug: dossier.slug.clone() },
            class: "browse-card",
            div { class: "browse-card-header",
                h3 { class: "browse-card-title", "{display}" }
                span { class: "browse-card-count", "{dossier.chunk_count}" }
            }
            div { class: "browse-bar-track",
                div {
                    class: "browse-bar-fill",
                    style: "width: {pct}%;",
                }
            }
            div { class: "browse-card-meta",
                if !date_label.is_empty() {
                    span { class: "browse-meta-item", "{date_label}" }
                }
                span { class: "browse-meta-item",
                    "{dossier.document_count} documents"
                }
                for (author, _) in dossier.authors.iter().take(3) {
                    span { class: "browse-meta-chip", "{author}" }
                }
            }
        }
    }
}

#[component]
pub fn CollectionPage(slug: String) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    if state.loading {
        return rsx! { LoadingSpinner { message: "Loading...".to_string() } };
    }
    let Some(archive) = state.archive.as_ref() else {
        return rsx! { p { class: "card-empty", "Archive not loaded." } };
    };

    let dossiers = build_collection_dossiers(archive.chunks());
    let Some(dossier) = find_collection(&dossiers, &slug) else {
        return rsx! {
            div { class: "page",
                h2 { "Collection not found" }
                p { "No collection matches \"{slug}\"." }
                Link { to: Route::CollectionsIndexPage {}, "Back to all collections" }
            }
        };
    };

    let date_label = dossier
        .date_range
        .as_ref()
        .map(|(a, b)| {
            if a == b {
                a.clone()
            } else {
                format!("{a} – {b}")
            }
        })
        .unwrap_or_else(|| "no dates".to_string());

    let display_name = dossier.name.replace('_', " ");
    let samples: Vec<constitution_archive::Chunk> = dossier
        .sample_chunk_ids
        .iter()
        .filter_map(|id| archive.chunk(id).ok().cloned())
        .collect();

    rsx! {
        div { class: "page browse-detail-page",
            header { class: "page-header",
                div { class: "browse-breadcrumb",
                    Link { to: Route::CollectionsIndexPage {}, "All collections" }
                    span { class: "breadcrumb-sep", " / " }
                    span { "{display_name}" }
                }
                div { class: "page-header-row",
                    div {
                        h2 { "{display_name}" }
                        p { class: "page-subtitle",
                            "{dossier.chunk_count} chunks · {dossier.document_count} documents · {date_label}"
                        }
                    }
                    PermalinkButton { label: Some("Share".to_string()) }
                }
            }

            div { class: "browse-detail-grid",
                section { class: "browse-detail-card",
                    h4 { "Top contributors" }
                    if dossier.authors.is_empty() {
                        p { class: "card-empty", "No author metadata." }
                    } else {
                        ul { class: "browse-key-list",
                            for (author, n) in dossier.authors.iter() {
                                li {
                                    Link {
                                        to: Route::AuthorPage { slug: crate::components::browse::slugify(author) },
                                        class: "browse-key-link",
                                        span { class: "browse-key-name", "{author}" }
                                        span { class: "browse-key-count", "{n}" }
                                    }
                                }
                            }
                        }
                    }
                }
                section { class: "browse-detail-card",
                    h4 { "Top issues" }
                    if dossier.top_issues.is_empty() {
                        p { class: "card-empty", "No issue tags recorded." }
                    } else {
                        div { class: "browse-tag-cloud",
                            for (tag, n) in dossier.top_issues.iter() {
                                span { class: "browse-tag",
                                    "{tag}"
                                    span { class: "browse-tag-count", "{n}" }
                                }
                            }
                        }
                    }
                }
            }

            if !samples.is_empty() {
                section { class: "browse-samples",
                    h3 { "Representative passages" }
                    div { class: "siblings-grid",
                        for sample in samples.iter() {
                            Link {
                                to: Route::DocumentPage { id: sample.chunk_id.clone() },
                                class: "sibling-card",
                                div { class: "sibling-id", "{sample.title}" }
                                p { class: "sibling-preview", "{sample.ensured_preview()}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
