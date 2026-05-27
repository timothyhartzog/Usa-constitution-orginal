use dioxus::prelude::*;

use crate::router::Route;

const NAV_ENTRIES: &[(&str, &str, fn() -> Route)] = &[
    ("Dashboard", "~", || Route::DashboardPage {}),
    ("Search", "Q", || Route::SearchPage {}),
    ("Graph", "G", || Route::GraphPage {}),
    ("World", "W", || Route::WorldPage {}),
    ("Timeline", "T", || Route::TimelinePage {}),
    ("Blog", "B", || Route::BlogIndexPage {}),
];

#[component]
pub fn Sidebar() -> Element {
    let current_route: Route = use_route();

    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar-header",
                h1 { class: "sidebar-title", "Constitution Archive" }
                p { class: "sidebar-subtitle", "Research Workbench" }
            }
            nav { class: "sidebar-nav",
                for &(label, icon, route_fn) in NAV_ENTRIES {
                    { let route = route_fn();
                    let active = is_active(&current_route, &route);
                    rsx! {
                        Link {
                            to: route,
                            class: if active { "nav-item nav-item-active" } else { "nav-item" },
                            span { class: "nav-icon", "{icon}" }
                            span { class: "nav-label", "{label}" }
                        }
                    }}
                }
            }
            div { class: "sidebar-footer",
                p { class: "sidebar-version", "v0.1.0 WASM" }
            }
        }
    }
}

fn is_active(current: &Route, target: &Route) -> bool {
    match (current, target) {
        (Route::DashboardPage {}, Route::DashboardPage {}) => true,
        (Route::SearchPage {}, Route::SearchPage {}) => true,
        (Route::GraphPage {}, Route::GraphPage {}) => true,
        (Route::WorldPage {}, Route::WorldPage {}) => true,
        (Route::TimelinePage {}, Route::TimelinePage {}) => true,
        (Route::BlogIndexPage {}, Route::BlogIndexPage {})
        | (Route::BlogPostPage { .. }, Route::BlogIndexPage {})
        | (Route::BlogEditorPage {}, Route::BlogIndexPage {}) => true,
        (Route::DocumentPage { .. }, Route::SearchPage {}) => true,
        _ => false,
    }
}
