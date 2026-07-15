//! Cmd+K / Ctrl+K command palette. Lets the user fuzzy-jump to any page,
//! recent document, recent search, or built-in action without leaving
//! the keyboard. Lists are ranked by lightweight substring scoring.

use dioxus::prelude::*;

use crate::router::Route;
use crate::state::{
    use_archive, use_command_palette, use_search_state, use_theme, use_user_data,
    CommandPaletteState, Theme,
};
use crate::storage;

#[derive(Clone, Debug)]
struct Action {
    label: String,
    detail: String,
    /// Sort key for "no-query" listing.
    section: ActionSection,
    /// What to do when invoked.
    kind: ActionKind,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
enum ActionSection {
    Navigate,
    RecentDoc,
    Bookmark,
    RecentSearch,
    Setting,
}

impl ActionSection {
    fn label(self) -> &'static str {
        match self {
            Self::Navigate => "Navigate",
            Self::RecentDoc => "Recent docs",
            Self::Bookmark => "Bookmarks",
            Self::RecentSearch => "Recent searches",
            Self::Setting => "Settings",
        }
    }
}

#[derive(Clone, Debug)]
enum ActionKind {
    Route(Route),
    OpenDoc(String),
    RunSearch(String),
    CycleTheme,
    CopyPermalink,
}

fn build_actions() -> Vec<Action> {
    let user = use_user_data();
    let user = user.read();
    let theme = use_theme();
    let theme_label = match *theme.read() {
        Theme::System => "Cycle theme (System -> Light)",
        Theme::Light => "Cycle theme (Light -> Dark)",
        Theme::Dark => "Cycle theme (Dark -> System)",
    };

    let mut out: Vec<Action> = Vec::new();

    // Navigation routes
    let nav_routes: &[(&str, &str, fn() -> Route)] = &[
        (
            "Dashboard",
            "Coordinated view of search, graph, timeline, map",
            || Route::DashboardPage {},
        ),
        ("Search", "Full-text search with filters", || {
            Route::SearchPage {}
        }),
        ("Graph", "Force-directed citation network", || {
            Route::GraphPage {}
        }),
        ("World map", "194 world constitutions", || {
            Route::WorldPage {}
        }),
        ("Timeline", "Constitutional process events", || {
            Route::TimelinePage {}
        }),
        ("PDSA", "Plan Do Study Act improvement cycles", || {
            Route::PlanDoStudyActPage {}
        }),
        ("Blog", "Analysis posts with embedded widgets", || {
            Route::BlogIndexPage {}
        }),
        ("Blog editor", "Write a new post", || {
            Route::BlogEditorPage {}
        }),
    ];
    for &(label, detail, route_fn) in nav_routes {
        out.push(Action {
            label: label.to_string(),
            detail: detail.to_string(),
            section: ActionSection::Navigate,
            kind: ActionKind::Route(route_fn()),
        });
    }

    for b in &user.bookmarks {
        out.push(Action {
            label: b.title.clone(),
            detail: format!("Bookmark · {}", b.collection),
            section: ActionSection::Bookmark,
            kind: ActionKind::OpenDoc(b.chunk_id.clone()),
        });
    }

    for h in &user.history {
        out.push(Action {
            label: h.title.clone(),
            detail: format!("Recent · {}", h.collection),
            section: ActionSection::RecentDoc,
            kind: ActionKind::OpenDoc(h.chunk_id.clone()),
        });
    }

    for q in &user.recent_searches {
        out.push(Action {
            label: q.clone(),
            detail: "Recent search".to_string(),
            section: ActionSection::RecentSearch,
            kind: ActionKind::RunSearch(q.clone()),
        });
    }

    out.push(Action {
        label: "Copy permalink to this view".to_string(),
        detail: "Encodes the current selection + search into the URL".to_string(),
        section: ActionSection::Setting,
        kind: ActionKind::CopyPermalink,
    });

    out.push(Action {
        label: theme_label.to_string(),
        detail: "Toggle light / dark / system".to_string(),
        section: ActionSection::Setting,
        kind: ActionKind::CycleTheme,
    });

    out
}

fn score(text: &str, query: &str) -> i32 {
    if query.is_empty() {
        return 0;
    }
    let q = query.to_lowercase();
    let t = text.to_lowercase();
    if t == q {
        return 1_000;
    }
    if t.starts_with(&q) {
        return 600 - (t.len() as i32 - q.len() as i32).max(0);
    }
    if t.contains(&q) {
        return 300;
    }
    // Loose subsequence match
    let mut q_iter = q.chars().peekable();
    for c in t.chars() {
        if let Some(&qc) = q_iter.peek() {
            if c == qc {
                q_iter.next();
            }
        }
    }
    if q_iter.peek().is_none() {
        100
    } else {
        -1
    }
}

#[component]
pub fn CommandPalette() -> Element {
    let mut palette = use_command_palette();
    let mut highlight = use_signal(|| 0usize);
    let actions = build_actions();
    let navigator = use_navigator();
    let mut search_state = use_search_state();
    let mut theme = use_theme();

    let open = palette.read().open;
    if !open {
        return rsx! { div { class: "command-palette-root" } };
    }

    let query = palette.read().query.clone();
    let q_lower = query.to_lowercase();

    let mut ranked: Vec<(i32, &Action)> = actions
        .iter()
        .filter_map(|a| {
            if query.is_empty() {
                Some((0, a))
            } else {
                let label_s = score(&a.label, &q_lower);
                let detail_s = score(&a.detail, &q_lower) / 2;
                let best = label_s.max(detail_s);
                if best < 0 {
                    None
                } else {
                    Some((best, a))
                }
            }
        })
        .collect();

    if query.is_empty() {
        // Stable order: Navigate, then recent docs, bookmarks, searches, settings.
        ranked.sort_by_key(|(_, a)| match a.section {
            ActionSection::Navigate => 0,
            ActionSection::RecentDoc => 1,
            ActionSection::Bookmark => 2,
            ActionSection::RecentSearch => 3,
            ActionSection::Setting => 4,
        });
    } else {
        ranked.sort_by(|a, b| b.0.cmp(&a.0));
    }

    let visible: Vec<&Action> = ranked.iter().map(|(_, a)| *a).take(40).collect();
    let total = visible.len();
    let cur = (*highlight.read()).min(total.saturating_sub(1));

    let run_action = move |kind: ActionKind| {
        match kind {
            ActionKind::Route(r) => {
                navigator.push(r);
            }
            ActionKind::OpenDoc(id) => {
                navigator.push(Route::DocumentPage { id });
            }
            ActionKind::RunSearch(q) => {
                {
                    let mut ss = search_state.write();
                    ss.query = q.clone();
                }
                if let Some(ref archive) = use_archive().read().archive {
                    let opts = constitution_archive::SearchOptions {
                        limit: 50,
                        fuzzy_distance: 1,
                        snippet_window: 240,
                        ..Default::default()
                    };
                    let results =
                        archive.search(&q, &constitution_archive::Filter::default(), &opts);
                    let mut ss = search_state.write();
                    ss.total_results = results.len();
                    ss.results = results;
                }
                navigator.push(Route::SearchPage {});
            }
            ActionKind::CycleTheme => {
                let next = theme.read().next();
                theme.set(next);
                storage::set(storage::KEY_THEME, next.as_str());
            }
            ActionKind::CopyPermalink => {
                use crate::components::url_sync::{copy_to_clipboard, current_share_url};
                let url = current_share_url();
                if !url.is_empty() {
                    spawn(async move {
                        let _ = copy_to_clipboard(&url).await;
                    });
                }
            }
        }
        palette.set(CommandPaletteState::default());
    };

    rsx! {
        div {
            class: "command-palette-overlay",
            role: "dialog",
            aria_modal: "true",
            aria_label: "Command palette",
            onclick: move |_| palette.set(CommandPaletteState::default()),
            div {
                class: "command-palette",
                onclick: move |e| e.stop_propagation(),
                div { class: "cp-input-row",
                    span { class: "cp-prompt", ">" }
                    input {
                        class: "cp-input",
                        r#type: "text",
                        autofocus: true,
                        placeholder: "Type a command, page, doc, or search...",
                        value: "{query}",
                        oninput: move |e| {
                            palette.write().query = e.value();
                            highlight.set(0);
                        },
                        onkeydown: {
                            let visible = visible.iter().map(|a| (*a).clone()).collect::<Vec<_>>();
                            let mut run_action = run_action.clone();
                            let mut palette = palette;
                            move |e: KeyboardEvent| {
                                match e.key() {
                                    Key::Escape => {
                                        palette.set(CommandPaletteState::default());
                                    }
                                    Key::ArrowDown => {
                                        e.prevent_default();
                                        let mut h = highlight.write();
                                        if total > 0 { *h = (*h + 1) % total; }
                                    }
                                    Key::ArrowUp => {
                                        e.prevent_default();
                                        let mut h = highlight.write();
                                        if total > 0 {
                                            *h = if *h == 0 { total - 1 } else { *h - 1 };
                                        }
                                    }
                                    Key::Enter => {
                                        if let Some(action) = visible.get(cur) {
                                            run_action(action.kind.clone());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        },
                    }
                    button {
                        class: "cp-close",
                        aria_label: "Close palette",
                        onclick: move |_| palette.set(CommandPaletteState::default()),
                        "Esc"
                    }
                }
                if visible.is_empty() {
                    div { class: "cp-empty", "No matches" }
                } else {
                    ul {
                        class: "cp-list",
                        role: "listbox",
                        for (i, action) in visible.iter().enumerate() {
                            {
                                let kind = action.kind.clone();
                                let mut run_action = run_action.clone();
                                let is_current = i == cur;
                                let section = action.section.label();
                                rsx! {
                                    li {
                                        role: "option",
                                        aria_selected: if is_current { "true" } else { "false" },
                                        class: if is_current { "cp-item cp-item-current" } else { "cp-item" },
                                        onclick: move |_| run_action(kind.clone()),
                                        onmouseenter: move |_| highlight.set(i),
                                        div { class: "cp-item-main",
                                            div { class: "cp-item-label", "{action.label}" }
                                            div { class: "cp-item-detail", "{action.detail}" }
                                        }
                                        span { class: "cp-item-section", "{section}" }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "cp-footer",
                    span { class: "cp-hint",
                        kbd { "Enter" }
                        " open · "
                        kbd { "↑↓" }
                        " navigate · "
                        kbd { "Esc" }
                        " close"
                    }
                    span { class: "cp-hint", "{visible.len()} results" }
                }
            }
        }
    }
}
