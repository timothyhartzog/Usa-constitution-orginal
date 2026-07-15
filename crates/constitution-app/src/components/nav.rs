use dioxus::prelude::*;

use crate::router::Route;
use crate::state::{use_command_palette, use_shortcuts, use_theme, CommandPaletteState, Theme};
use crate::storage;

const NAV_ENTRIES: &[(&str, &str, fn() -> Route)] = &[
    ("Dashboard", "~", || Route::DashboardPage {}),
    ("Search", "Q", || Route::SearchPage {}),
    ("Chat", "💬", || Route::ChatPage {}),
    ("Authors", "A", || Route::AuthorsIndexPage {}),
    ("Collections", "C", || Route::CollectionsIndexPage {}),
    ("Graph", "G", || Route::GraphPage {}),
    ("Network", "N", || Route::KnowledgeGraphPage {}),
    ("World", "W", || Route::WorldPage {}),
    ("Timeline", "T", || Route::TimelinePage {}),
    ("PDSA", "P", || Route::PlanDoStudyActPage {}),
    ("Blog", "B", || Route::BlogIndexPage {}),
];

#[component]
pub fn Sidebar() -> Element {
    let current_route: Route = use_route();
    let mut sidebar_open = use_signal(|| false);

    rsx! {
        // Mobile burger
        button {
            class: "sidebar-burger",
            onclick: move |_| {
                let v = *sidebar_open.read();
                sidebar_open.set(!v);
            },
            if *sidebar_open.read() { "x" } else { "=" }
        }
        aside {
            class: if *sidebar_open.read() { "sidebar sidebar-open" } else { "sidebar" },
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
                            onclick: move |_| sidebar_open.set(false),
                            span { class: "nav-icon", "{icon}" }
                            span { class: "nav-label", "{label}" }
                        }
                    }}
                }
            }
            div { class: "sidebar-footer",
                CommandPaletteButton {}
                ThemeToggle {}
                ShortcutsHelpButton {}
                p { class: "sidebar-version", "v0.1.0 WASM" }
            }
        }
    }
}

#[component]
fn ThemeToggle() -> Element {
    let mut theme = use_theme();

    let current = *theme.read();
    let next = current.next();
    let (label, glyph) = match current {
        Theme::System => ("System", "*"),
        Theme::Light => ("Light", "+"),
        Theme::Dark => ("Dark", "."),
    };

    rsx! {
        button {
            class: "theme-toggle",
            title: "Cycle theme (current: {label})",
            onclick: move |_| {
                let new_theme = next;
                theme.set(new_theme);
                storage::set(storage::KEY_THEME, new_theme.as_str());
            },
            span { class: "theme-toggle-glyph", "{glyph}" }
            span { class: "theme-toggle-label", "{label}" }
        }
    }
}

#[component]
fn CommandPaletteButton() -> Element {
    let mut palette = use_command_palette();
    rsx! {
        button {
            class: "command-palette-trigger",
            title: "Open command palette (Cmd+K)",
            aria_label: "Open command palette",
            onclick: move |_| palette.set(CommandPaletteState { open: true, query: String::new() }),
            span { class: "cp-trigger-icon", "K" }
            span { class: "cp-trigger-label", "Command palette" }
            span { class: "cp-trigger-hint",
                kbd { "⌘" }
                kbd { "K" }
            }
        }
    }
}

#[component]
fn ShortcutsHelpButton() -> Element {
    let mut shortcuts = use_shortcuts();
    rsx! {
        button {
            class: "shortcuts-help-trigger",
            title: "Show keyboard shortcuts (?)",
            aria_label: "Show keyboard shortcuts",
            onclick: move |_| shortcuts.write().help_open = true,
            span { class: "cp-trigger-icon", "?" }
            span { class: "cp-trigger-label", "Keyboard shortcuts" }
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
        (Route::PlanDoStudyActPage {}, Route::PlanDoStudyActPage {}) => true,
        (Route::AuthorsIndexPage {}, Route::AuthorsIndexPage {})
        | (Route::AuthorPage { .. }, Route::AuthorsIndexPage {}) => true,
        (Route::CollectionsIndexPage {}, Route::CollectionsIndexPage {})
        | (Route::CollectionPage { .. }, Route::CollectionsIndexPage {}) => true,
        (Route::BlogIndexPage {}, Route::BlogIndexPage {})
        | (Route::BlogPostPage { .. }, Route::BlogIndexPage {})
        | (Route::BlogEditorPage {}, Route::BlogIndexPage {}) => true,
        (Route::DocumentPage { .. }, Route::SearchPage {}) => true,
        _ => false,
    }
}
