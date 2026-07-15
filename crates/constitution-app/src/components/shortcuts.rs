//! Global keyboard shortcuts. Implemented as an invisible Dioxus
//! component that registers a window-level keydown listener (web) and
//! routes the event to the global signals.
//!
//! Shortcuts:
//!   Cmd/Ctrl+K   : open command palette
//!   /            : focus the search bar (any page)
//!   ?            : show the shortcuts help modal
//!   Esc          : close palette / help / popovers
//!   g d          : go to dashboard (chord)
//!   g s          : go to search
//!   g g          : go to graph
//!   g w          : go to world map
//!   g t          : go to timeline
//!   g p          : go to PDSA
//!   g b          : go to blog index
//!
//! Inputs (input / textarea) are excluded so typing in a field doesn't
//! trigger navigation.

use dioxus::prelude::*;

use crate::router::Route;
use crate::state::{use_command_palette, use_shortcuts, CommandPaletteState};

#[component]
pub fn GlobalShortcuts() -> Element {
    let palette = use_command_palette();
    let shortcuts = use_shortcuts();
    let navigator = use_navigator();

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;

        use_hook(move || {
            let Some(window) = web_sys::window() else {
                return;
            };

            let mut palette_for_cb = palette;
            let mut shortcuts_for_cb = shortcuts;
            let navigator_for_cb = navigator;

            let cb = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                let key = event.key();
                let meta = event.meta_key() || event.ctrl_key();

                // Cmd/Ctrl+K opens the palette regardless of focus.
                if meta && (key == "k" || key == "K") {
                    event.prevent_default();
                    let cur = palette_for_cb.read().open;
                    if cur {
                        palette_for_cb.set(CommandPaletteState::default());
                    } else {
                        palette_for_cb.set(CommandPaletteState {
                            open: true,
                            query: String::new(),
                        });
                    }
                    return;
                }

                // Esc closes overlays from anywhere
                if key == "Escape" {
                    let mut anything_was_open = false;
                    if palette_for_cb.read().open {
                        palette_for_cb.set(CommandPaletteState::default());
                        anything_was_open = true;
                    }
                    if shortcuts_for_cb.read().help_open {
                        shortcuts_for_cb.write().help_open = false;
                        anything_was_open = true;
                    }
                    if anything_was_open {
                        event.prevent_default();
                    }
                    return;
                }

                // If the user is typing into an editable element, ignore
                // single-key chords / nav so they don't fight with text input.
                let target = event.target();
                let editable = target
                    .as_ref()
                    .and_then(|t| t.dyn_ref::<web_sys::HtmlElement>())
                    .map(|el| is_editable(el))
                    .unwrap_or(false);

                if !editable {
                    if key == "/" {
                        event.prevent_default();
                        // Move focus to the first .search-input on the page.
                        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                            if let Ok(Some(el)) = doc.query_selector(
                                ".search-input, .blog-search-input, .world-search-input",
                            ) {
                                if let Some(input) = el.dyn_ref::<web_sys::HtmlElement>() {
                                    let _ = input.focus();
                                }
                            }
                        }
                        return;
                    }
                    if key == "?" {
                        event.prevent_default();
                        let cur = shortcuts_for_cb.read().help_open;
                        shortcuts_for_cb.write().help_open = !cur;
                        return;
                    }

                    // Chord support: g + <letter>
                    if key == "g" || key == "G" {
                        shortcuts_for_cb.write().pending_g = true;
                        return;
                    }

                    let pending = shortcuts_for_cb.read().pending_g;
                    if pending {
                        shortcuts_for_cb.write().pending_g = false;
                        let route = match key.as_str() {
                            "d" | "D" => Some(Route::DashboardPage {}),
                            "s" | "S" => Some(Route::SearchPage {}),
                            "g" | "G" => Some(Route::GraphPage {}),
                            "w" | "W" => Some(Route::WorldPage {}),
                            "t" | "T" => Some(Route::TimelinePage {}),
                            "p" | "P" => Some(Route::PlanDoStudyActPage {}),
                            "b" | "B" => Some(Route::BlogIndexPage {}),
                            _ => None,
                        };
                        if let Some(route) = route {
                            event.prevent_default();
                            navigator_for_cb.push(route);
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);

            let _ = window.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
            cb.forget(); // keep the closure alive for the app lifetime
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = (palette, shortcuts, navigator);

    rsx! {
        ShortcutsHelp {}
    }
}

#[cfg(target_arch = "wasm32")]
fn is_editable(el: &web_sys::HtmlElement) -> bool {
    let tag = el.tag_name().to_uppercase();
    if tag == "INPUT" || tag == "TEXTAREA" || tag == "SELECT" {
        return true;
    }
    el.is_content_editable()
}

#[component]
fn ShortcutsHelp() -> Element {
    let mut shortcuts = use_shortcuts();
    let open = shortcuts.read().help_open;

    if !open {
        return rsx! { div { class: "shortcuts-help-root" } };
    }

    rsx! {
        div {
            class: "shortcuts-help-overlay",
            role: "dialog",
            aria_modal: "true",
            aria_label: "Keyboard shortcuts",
            onclick: move |_| shortcuts.write().help_open = false,
            div { class: "shortcuts-help",
                onclick: move |e| e.stop_propagation(),
                div { class: "shortcuts-help-header",
                    h3 { "Keyboard shortcuts" }
                    button {
                        class: "modal-close",
                        aria_label: "Close",
                        onclick: move |_| shortcuts.write().help_open = false,
                        "x"
                    }
                }
                div { class: "shortcuts-help-body",
                    section { class: "shortcuts-group",
                        h4 { "Global" }
                        Row { keys: vec!["Cmd".to_string(), "K".to_string()], desc: "Open command palette" }
                        Row { keys: vec!["Ctrl".to_string(), "K".to_string()], desc: "Open command palette (Windows / Linux)" }
                        Row { keys: vec!["/".to_string()], desc: "Focus search input on the current page" }
                        Row { keys: vec!["?".to_string()], desc: "Show this help" }
                        Row { keys: vec!["Esc".to_string()], desc: "Close palette, help, or any modal" }
                    }
                    section { class: "shortcuts-group",
                        h4 { "Navigation (chord)" }
                        Row { keys: vec!["g".to_string(), "d".to_string()], desc: "Dashboard" }
                        Row { keys: vec!["g".to_string(), "s".to_string()], desc: "Search" }
                        Row { keys: vec!["g".to_string(), "g".to_string()], desc: "Citation graph" }
                        Row { keys: vec!["g".to_string(), "w".to_string()], desc: "World constitutions" }
                        Row { keys: vec!["g".to_string(), "t".to_string()], desc: "Timeline" }
                        Row { keys: vec!["g".to_string(), "p".to_string()], desc: "PDSA" }
                        Row { keys: vec!["g".to_string(), "b".to_string()], desc: "Blog" }
                    }
                    section { class: "shortcuts-group",
                        h4 { "Document reader" }
                        Row { keys: vec!["b".to_string()], desc: "Toggle bookmark for the current document" }
                    }
                }
            }
        }
    }
}

#[component]
fn Row(keys: Vec<String>, desc: String) -> Element {
    rsx! {
        div { class: "shortcuts-row",
            div { class: "shortcuts-keys",
                for (i, k) in keys.iter().enumerate() {
                    {
                        let key = format!("k-{i}");
                        rsx! {
                            kbd { key: "{key}", "{k}" }
                            if i + 1 < keys.len() {
                                span { class: "shortcuts-plus", "+" }
                            }
                        }
                    }
                }
            }
            div { class: "shortcuts-desc", "{desc}" }
        }
    }
}
