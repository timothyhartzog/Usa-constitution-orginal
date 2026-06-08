//! Sync selection + search state to/from the URL fragment so views are
//! shareable via copy-link. Mounted once near the app root.
//!
//! Direction of authority:
//!   - On mount: URL -> state (restore from a deep link)
//!   - On state change: state -> URL (replaceState; no extra history)

use dioxus::prelude::*;

use crate::state::{use_search_state, use_selection};
use crate::url_state::{decode, encode, ShareState};

#[component]
pub fn UrlSync() -> Element {
    let mut selection = use_selection();
    let mut search_state = use_search_state();

    // 1. Hydrate from URL on first mount.
    use_hook(move || {
        let frag = current_fragment();
        if frag.is_empty() {
            return;
        }
        let share = decode(&frag);
        if let Some(sel) = share.selection.clone() {
            selection.set(crate::state::SelectionState { kind: sel });
        }
        if !share.query.is_empty()
            || !share.collections.is_empty()
            || !share.issues.is_empty()
            || share.date_prefix.is_some()
        {
            let filter = build_filter(&share);
            let mut ss = search_state.write();
            ss.query = share.query;
            ss.filter = filter;
        }
    });

    // 2. Write back whenever selection / search change.
    let selection_kind = selection.read().kind.clone();
    let search_query = search_state.read().query.clone();
    let search_filter = search_state.read().filter.clone();

    use_effect(use_reactive!(|(selection_kind, search_query, search_filter)| {
        let share = ShareState::from_selection_and_search(
            &crate::state::SelectionState { kind: selection_kind.clone() },
            &search_query,
            &search_filter,
        );
        let frag = encode(&share);
        set_fragment(&frag);
    }));

    rsx! { div { class: "url-sync-root", style: "display:none" } }
}

fn build_filter(share: &ShareState) -> constitution_archive::Filter {
    use constitution_archive::{Filter, FilterValue};
    let mut filter = Filter::default();
    if !share.collections.is_empty() {
        filter = filter.with(FilterValue::Collection(share.collections.clone()));
    }
    if !share.issues.is_empty() {
        filter = filter.with(FilterValue::IssueTag(share.issues.clone()));
    }
    if !share.authors.is_empty() {
        filter = filter.with(FilterValue::Author(share.authors.clone()));
    }
    if !share.doc_types.is_empty() {
        filter = filter.with(FilterValue::DocumentType(share.doc_types.clone()));
    }
    if !share.clauses.is_empty() {
        filter = filter.with(FilterValue::ClauseTag(share.clauses.clone()));
    }
    if let Some(d) = &share.date_prefix {
        filter = filter.with(FilterValue::DatePrefix(d.clone()));
    }
    filter
}

#[cfg(target_arch = "wasm32")]
fn current_fragment() -> String {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn current_fragment() -> String {
    String::new()
}

#[cfg(target_arch = "wasm32")]
fn set_fragment(frag: &str) {
    let Some(window) = web_sys::window() else { return };
    let Ok(history) = window.history() else { return };
    let Ok(location) = web_sys::window().map(|w| w.location()).ok_or(()) else { return };
    let path = location.pathname().unwrap_or_default();
    let search = location.search().unwrap_or_default();
    let new_url = if frag.is_empty() {
        format!("{path}{search}")
    } else {
        format!("{path}{search}#{frag}")
    };
    // replaceState (vs pushState) so we don't pollute the back/forward
    // stack on every keystroke in the search box.
    let _ = history.replace_state_with_url(
        &wasm_bindgen::JsValue::NULL,
        "",
        Some(&new_url),
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn set_fragment(_frag: &str) {}

/// Returns the full URL (origin + path + search + hash) for sharing.
pub fn current_share_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.location().href().ok())
            .unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        String::new()
    }
}

/// Copy a string to the system clipboard using the modern async API.
/// Returns true on success.
pub async fn copy_to_clipboard(text: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen_futures::JsFuture;
        let Some(window) = web_sys::window() else { return false };
        let navigator = window.navigator();
        let clipboard = navigator.clipboard();
        let promise = clipboard.write_text(text);
        JsFuture::from(promise).await.is_ok()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = text;
        false
    }
}
