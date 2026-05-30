use std::rc::Rc;

use constitution_archive::{Archive, ArchiveStats, Chunk, Filter, SearchHit, SearchOptions};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorldConstitutionMeta {
    pub document_id: String,
    pub constitute_id: String,
    pub country_id: String,
    pub country: String,
    pub region: String,
    pub status: String,
    pub word_count: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ArchiveState {
    pub archive: Option<Rc<Archive>>,
    pub world_meta: Vec<WorldConstitutionMeta>,
    pub loading: bool,
    pub error: Option<String>,
    /// 0..=100 download progress for the archive. 100 = fully fetched
    /// (parsing happens synchronously afterward). 0 if no Content-Length.
    pub progress_percent: u8,
    /// Last-known bytes fetched (for display).
    pub bytes_fetched: u64,
    /// Content-Length if known.
    pub bytes_total: u64,
}

impl ArchiveState {
    pub fn stats(&self) -> Option<ArchiveStats> {
        self.archive.as_ref().map(|a| a.stats())
    }

    pub fn search(&self, query: &str, filter: &Filter, opts: &SearchOptions) -> Vec<SearchHit> {
        self.archive
            .as_ref()
            .map(|a| a.search(query, filter, opts))
            .unwrap_or_default()
    }

    pub fn chunk(&self, id: &str) -> Option<Chunk> {
        self.archive
            .as_ref()
            .and_then(|a| a.chunk(id).ok().cloned())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum SelectionKind {
    #[default]
    None,
    Clause(String),
    Person(String),
    Essay(String),
    Country(String),
    #[allow(dead_code)]
    Chunk(String),
}

#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub kind: SelectionKind,
}

impl SelectionState {
    #[allow(dead_code)]
    pub fn select_clause(key: String) -> Self {
        Self {
            kind: SelectionKind::Clause(key),
        }
    }

    #[allow(dead_code)]
    pub fn select_person(key: String) -> Self {
        Self {
            kind: SelectionKind::Person(key),
        }
    }

    pub fn select_country(key: String) -> Self {
        Self {
            kind: SelectionKind::Country(key),
        }
    }

    pub fn clear() -> Self {
        Self {
            kind: SelectionKind::None,
        }
    }

    pub fn target_key(&self) -> Option<String> {
        match &self.kind {
            SelectionKind::None => None,
            SelectionKind::Clause(k) => Some(format!("clause:{k}")),
            SelectionKind::Person(k) => Some(format!("person:{k}")),
            SelectionKind::Essay(k) => Some(format!("essay:{k}")),
            SelectionKind::Country(k) => Some(k.clone()),
            SelectionKind::Chunk(k) => Some(k.clone()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub query: String,
    pub results: Vec<SearchHit>,
    pub filter: Filter,
    pub total_results: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlogPost {
    pub slug: String,
    pub title: String,
    pub date: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub excerpt: String,
    pub html: String,
    /// Original markdown source (kept so user-published posts can be re-edited).
    #[serde(default)]
    pub markdown: String,
    /// Whether the post came from a Markdown file (built-in) vs. the in-browser editor.
    #[serde(default)]
    pub user_created: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlogDraft {
    pub title: String,
    pub markdown: String,
    pub tags: String,
}

#[derive(Debug, Clone, Default)]
pub struct BlogState {
    pub posts: Vec<BlogPost>,
    pub draft: BlogDraft,
    pub tag_filter: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    pub fn from_str(s: &str) -> Self {
        match s {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::System,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::System => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::System,
        }
    }
}

pub fn use_archive() -> Signal<ArchiveState> {
    use_context::<Signal<ArchiveState>>()
}

pub fn use_selection() -> Signal<SelectionState> {
    use_context::<Signal<SelectionState>>()
}

pub fn use_search_state() -> Signal<SearchState> {
    use_context::<Signal<SearchState>>()
}

pub fn use_theme() -> Signal<Theme> {
    use_context::<Signal<Theme>>()
}

pub fn use_blog() -> Signal<BlogState> {
    use_context::<Signal<BlogState>>()
}

/// Cap on the number of entries kept in user history / bookmarks. The
/// underlying localStorage budget is small (~5 MB), and at ~120 bytes
/// per entry these caps keep us well under any host's per-key quota.
pub const HISTORY_LIMIT: usize = 30;
pub const BOOKMARK_LIMIT: usize = 50;
pub const RECENT_SEARCH_LIMIT: usize = 12;

/// One viewed document.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    pub chunk_id: String,
    pub title: String,
    #[serde(default)]
    pub collection: String,
}

#[derive(Debug, Clone, Default)]
pub struct UserData {
    pub history: Vec<HistoryEntry>,
    pub bookmarks: Vec<HistoryEntry>,
    pub recent_searches: Vec<String>,
}

impl UserData {
    pub fn push_history(&mut self, entry: HistoryEntry) {
        self.history.retain(|e| e.chunk_id != entry.chunk_id);
        self.history.insert(0, entry);
        if self.history.len() > HISTORY_LIMIT {
            self.history.truncate(HISTORY_LIMIT);
        }
    }

    pub fn push_recent_search(&mut self, query: String) {
        let q = query.trim().to_string();
        if q.is_empty() {
            return;
        }
        self.recent_searches.retain(|x| x != &q);
        self.recent_searches.insert(0, q);
        if self.recent_searches.len() > RECENT_SEARCH_LIMIT {
            self.recent_searches.truncate(RECENT_SEARCH_LIMIT);
        }
    }

    pub fn toggle_bookmark(&mut self, entry: HistoryEntry) -> bool {
        let was_bookmarked = self.bookmarks.iter().any(|b| b.chunk_id == entry.chunk_id);
        if was_bookmarked {
            self.bookmarks.retain(|b| b.chunk_id != entry.chunk_id);
            false
        } else {
            self.bookmarks.insert(0, entry);
            if self.bookmarks.len() > BOOKMARK_LIMIT {
                self.bookmarks.truncate(BOOKMARK_LIMIT);
            }
            true
        }
    }

    pub fn is_bookmarked(&self, chunk_id: &str) -> bool {
        self.bookmarks.iter().any(|b| b.chunk_id == chunk_id)
    }
}

/// Persistent form of UserData written to localStorage.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserDataPersisted {
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
    #[serde(default)]
    pub bookmarks: Vec<HistoryEntry>,
    #[serde(default)]
    pub recent_searches: Vec<String>,
}

pub fn use_user_data() -> Signal<UserData> {
    use_context::<Signal<UserData>>()
}

#[derive(Debug, Clone, Default)]
pub struct CommandPaletteState {
    pub open: bool,
    pub query: String,
}

pub fn use_command_palette() -> Signal<CommandPaletteState> {
    use_context::<Signal<CommandPaletteState>>()
}

#[derive(Debug, Clone, Default)]
pub struct ShortcutsState {
    pub help_open: bool,
    /// Last keydown for chord-detection ("g" then "d" -> dashboard).
    pub pending_g: bool,
}

pub fn use_shortcuts() -> Signal<ShortcutsState> {
    use_context::<Signal<ShortcutsState>>()
}
