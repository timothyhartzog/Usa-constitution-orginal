use constitution_archive::{Filter, FilterValue};
use dioxus::prelude::*;

use crate::state::use_search_state;

const COLLECTIONS: &[&str] = &[
    "constitution",
    "federalist_papers",
    "anti_federalist",
    "madisons_notes",
    "farrands_records",
    "elliots_debates",
    "founders_correspondence",
    "bill_of_rights",
    "letters_delegates_congress",
    "comparative_constitutions_world",
    "comparative_constitutions_eu",
];

const ISSUES: &[&str] = &[
    "federalism",
    "separation_of_powers",
    "individual_rights",
    "representation",
    "executive_power",
    "judicial_review",
    "amendment_process",
    "commerce",
    "taxation",
    "military",
];

#[component]
pub fn FilterPanel() -> Element {
    let mut search_state = use_search_state();
    let mut selected_collections = use_signal(Vec::<String>::new);
    let mut selected_issues = use_signal(Vec::<String>::new);
    let mut date_prefix = use_signal(String::new);

    let apply_filters = move |_| {
        let mut filter = Filter::default();
        let cols = selected_collections.read().clone();
        if !cols.is_empty() {
            filter = filter.with(FilterValue::Collection(cols));
        }
        let issues = selected_issues.read().clone();
        if !issues.is_empty() {
            filter = filter.with(FilterValue::IssueTag(issues));
        }
        let dp = date_prefix.read().clone();
        if !dp.is_empty() {
            filter = filter.with(FilterValue::DatePrefix(dp));
        }
        search_state.write().filter = filter;
    };

    rsx! {
        div { class: "filter-panel",
            h3 { class: "filter-title", "Filters" }

            div { class: "filter-group",
                h4 { "Collections" }
                for &col in COLLECTIONS {
                    label { class: "filter-checkbox",
                        input {
                            r#type: "checkbox",
                            onchange: {
                                let col = col.to_string();
                                move |e: Event<FormData>| {
                                    let checked = e.checked();
                                    let mut cols = selected_collections.write();
                                    if checked {
                                        cols.push(col.clone());
                                    } else {
                                        cols.retain(|c| c != &col);
                                    }
                                }
                            },
                        }
                        span { "{col}" }
                    }
                }
            }

            div { class: "filter-group",
                h4 { "Issues" }
                for &issue in ISSUES {
                    label { class: "filter-checkbox",
                        input {
                            r#type: "checkbox",
                            onchange: {
                                let issue = issue.to_string();
                                move |e: Event<FormData>| {
                                    let checked = e.checked();
                                    let mut issues = selected_issues.write();
                                    if checked {
                                        issues.push(issue.clone());
                                    } else {
                                        issues.retain(|i| i != &issue);
                                    }
                                }
                            },
                        }
                        span { "{issue}" }
                    }
                }
            }

            div { class: "filter-group",
                h4 { "Date" }
                input {
                    r#type: "text",
                    class: "filter-input",
                    placeholder: "e.g. 1787",
                    value: "{date_prefix}",
                    oninput: move |e| date_prefix.set(e.value()),
                }
            }

            button {
                class: "filter-apply",
                onclick: apply_filters,
                "Apply Filters"
            }
        }
    }
}
