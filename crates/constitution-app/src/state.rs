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

#[derive(Debug, Clone, Default)]
pub struct BlogPost {
    pub slug: String,
    pub title: String,
    pub date: String,
    pub tags: Vec<String>,
    pub excerpt: String,
    pub html: String,
}

#[derive(Debug, Clone, Default)]
pub struct BlogState {
    pub posts: Vec<BlogPost>,
    pub draft_markdown: String,
    pub draft_title: String,
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

pub fn use_blog() -> Signal<BlogState> {
    use_context::<Signal<BlogState>>()
}
