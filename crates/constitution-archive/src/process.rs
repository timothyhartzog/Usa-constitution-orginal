//! Process-timeline lookup: "what happened when, who did it, and which
//! source documents in the archive are about it".
//!
//! The timeline data lives in `data/process_timeline.json` and is bundled
//! into the binary archive at build time. Each event references zero or more
//! `chunk_id`s so the UI can pivot from "show me events about ratification
//! in Virginia" → the underlying primary-source passages.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::chunk::ChunkId;
use crate::error::ArchiveError;

/// Coarse phase of the constitutional process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessPhase {
    /// Pre-convention: failures of the Articles of Confederation.
    Antecedents,
    /// Constitutional Convention (May–September 1787).
    Convention,
    /// Public ratification debate (1787–1788).
    Ratification,
    /// First Federal Congress drafts the Bill of Rights (1789).
    BillOfRightsDrafting,
    /// State ratification of the Bill of Rights (1789–1791).
    BillOfRightsRatification,
}

impl ProcessPhase {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Antecedents => "Antecedents",
            Self::Convention => "Constitutional Convention",
            Self::Ratification => "Ratification debate",
            Self::BillOfRightsDrafting => "Bill of Rights drafting",
            Self::BillOfRightsRatification => "Bill of Rights ratification",
        }
    }
}

/// A single dated event in the constitutional process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    /// Stable slug identifier (e.g. `convention_great_compromise`).
    pub id: String,
    /// ISO-8601 calendar date or year/range.
    pub date: String,
    /// Phase classification.
    pub phase: ProcessPhase,
    /// One-line headline.
    pub title: String,
    /// Free-text description.
    pub summary: String,
    /// Key participants (names, free-form for now).
    #[serde(default)]
    pub actors: Vec<String>,
    /// Locations (city/state).
    #[serde(default)]
    pub locations: Vec<String>,
    /// Chunk identifiers in the archive that document this event.
    #[serde(default)]
    pub source_chunks: Vec<ChunkId>,
    /// Cross-references to other event ids (causally related).
    #[serde(default)]
    pub cross_refs: Vec<String>,
}

/// Ordered timeline of events plus an id index.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessTimeline {
    /// Events in chronological order (caller is responsible for sorting on build).
    pub events: Vec<ProcessEvent>,
    /// id → position in `events`.
    #[serde(default)]
    pub by_id: HashMap<String, u32>,
}

impl ProcessTimeline {
    /// Builds from a vector of events; sorts by `date` and rebuilds the id index.
    pub fn from_events(mut events: Vec<ProcessEvent>) -> Self {
        events.sort_by(|a, b| a.date.cmp(&b.date));
        let by_id = events
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id.clone(), i as u32))
            .collect();
        Self { events, by_id }
    }

    /// Loads from the JSON form shipped in `data/process_timeline.json`.
    pub fn from_json(bytes: &[u8]) -> Result<Self, ArchiveError> {
        let events: Vec<ProcessEvent> = serde_json::from_slice(bytes)?;
        Ok(Self::from_events(events))
    }

    /// Total event count.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns `true` if the timeline has no events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Look up an event by its slug.
    pub fn get(&self, id: &str) -> Result<&ProcessEvent, ArchiveError> {
        let pos = self
            .by_id
            .get(id)
            .ok_or_else(|| ArchiveError::ProcessEventNotFound(id.to_string()))?;
        self.events
            .get(*pos as usize)
            .ok_or_else(|| ArchiveError::ProcessEventNotFound(id.to_string()))
    }

    /// All events whose phase matches.
    pub fn by_phase(&self, phase: ProcessPhase) -> Vec<&ProcessEvent> {
        self.events.iter().filter(|e| e.phase == phase).collect()
    }

    /// All events whose `date` starts with `prefix` (e.g. `"1787"`).
    pub fn by_date_prefix(&self, prefix: &str) -> Vec<&ProcessEvent> {
        self.events
            .iter()
            .filter(|e| e.date.starts_with(prefix))
            .collect()
    }

    /// Free-text search across title/summary/actors/locations
    /// (case-insensitive substring match — coarse but adequate for ~50 events).
    pub fn search(&self, q: &str) -> Vec<&ProcessEvent> {
        let needle = q.to_lowercase();
        self.events
            .iter()
            .filter(|e| {
                e.title.to_lowercase().contains(&needle)
                    || e.summary.to_lowercase().contains(&needle)
                    || e.actors.iter().any(|a| a.to_lowercase().contains(&needle))
                    || e.locations.iter().any(|l| l.to_lowercase().contains(&needle))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(id: &str, date: &str, phase: ProcessPhase, title: &str) -> ProcessEvent {
        ProcessEvent {
            id: id.into(),
            date: date.into(),
            phase,
            title: title.into(),
            summary: format!("summary of {}", title),
            actors: vec!["Madison".into()],
            locations: vec!["Philadelphia".into()],
            source_chunks: vec![],
            cross_refs: vec![],
        }
    }

    #[test]
    fn sorts_and_indexes() {
        let t = ProcessTimeline::from_events(vec![
            ev("b", "1787-07-16", ProcessPhase::Convention, "Great Compromise"),
            ev("a", "1786-09-11", ProcessPhase::Antecedents, "Annapolis Convention"),
        ]);
        assert_eq!(t.events[0].id, "a");
        assert_eq!(t.events[1].id, "b");
        assert_eq!(t.get("b").unwrap().title, "Great Compromise");
    }

    #[test]
    fn phase_and_date_filters() {
        let t = ProcessTimeline::from_events(vec![
            ev("a", "1787-05-29", ProcessPhase::Convention, "Virginia Plan"),
            ev("b", "1788-06-21", ProcessPhase::Ratification, "Ninth State"),
        ]);
        assert_eq!(t.by_phase(ProcessPhase::Convention).len(), 1);
        assert_eq!(t.by_date_prefix("1788").len(), 1);
    }

    #[test]
    fn search_is_case_insensitive() {
        let t = ProcessTimeline::from_events(vec![ev(
            "x",
            "1787-09-17",
            ProcessPhase::Convention,
            "Signing of the Constitution",
        )]);
        assert_eq!(t.search("SIGNING").len(), 1);
        assert_eq!(t.search("madison").len(), 1);
        assert_eq!(t.search("paris").len(), 0);
    }
}
