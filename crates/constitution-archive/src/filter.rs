//! Metadata filtering used to narrow search results.

use serde::{Deserialize, Serialize};

use crate::chunk::Chunk;

/// A single faceted filter clause.
///
/// All clauses in a [`Filter`] must hold (logical AND); within a single clause
/// values are matched with logical OR. The matchers are case-insensitive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterValue {
    /// `chunk.source_collection in <values>`
    Collection(Vec<String>),
    /// `chunk.author contains any of <values>` (substring, case-insensitive)
    Author(Vec<String>),
    /// `chunk.document_type in <values>`
    DocumentType(Vec<String>),
    /// `chunk.issue_tags ∩ <values> ≠ ∅`
    IssueTag(Vec<String>),
    /// `chunk.constitutional_clause_tags ∩ <values> ≠ ∅`
    ClauseTag(Vec<String>),
    /// Date prefix match (e.g. `"1787"` matches all 1787 chunks).
    DatePrefix(String),
}

/// Conjunction of [`FilterValue`] clauses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Filter {
    /// Clauses combined with logical AND.
    pub clauses: Vec<FilterValue>,
}

impl Filter {
    /// Builder helper.
    pub fn with(mut self, value: FilterValue) -> Self {
        self.clauses.push(value);
        self
    }

    /// Returns `true` if the chunk satisfies every clause.
    pub fn matches(&self, chunk: &Chunk) -> bool {
        self.clauses.iter().all(|c| clause_matches(c, chunk))
    }
}

fn clause_matches(clause: &FilterValue, chunk: &Chunk) -> bool {
    match clause {
        FilterValue::Collection(values) => values
            .iter()
            .any(|v| v.eq_ignore_ascii_case(&chunk.source_collection)),
        FilterValue::Author(values) => {
            let author_l = chunk.author.to_lowercase();
            values.iter().any(|v| author_l.contains(&v.to_lowercase()))
        }
        FilterValue::DocumentType(values) => values
            .iter()
            .any(|v| v.eq_ignore_ascii_case(&chunk.document_type)),
        FilterValue::IssueTag(values) => values
            .iter()
            .any(|v| chunk.issue_tags.iter().any(|t| t.eq_ignore_ascii_case(v))),
        FilterValue::ClauseTag(values) => values.iter().any(|v| {
            chunk
                .constitutional_clause_tags
                .iter()
                .any(|t| t.eq_ignore_ascii_case(v))
        }),
        FilterValue::DatePrefix(prefix) => chunk.date.starts_with(prefix),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Chunk {
        Chunk {
            chunk_id: "x".into(),
            document_id: "x".into(),
            title: "t".into(),
            author: "James Madison".into(),
            date: "1787-09-17".into(),
            source_collection: "constitution".into(),
            source_url: "".into(),
            document_type: "foundational_document".into(),
            issue_tags: vec!["federalism".into()],
            constitutional_clause_tags: vec!["I.8".into()],
            text: "".into(),
            word_count: 0,
            preview: "".into(),
        }
    }

    #[test]
    fn and_of_clauses() {
        let f = Filter::default()
            .with(FilterValue::Collection(vec!["constitution".into()]))
            .with(FilterValue::Author(vec!["madison".into()]))
            .with(FilterValue::ClauseTag(vec!["I.8".into()]))
            .with(FilterValue::DatePrefix("1787".into()));
        assert!(f.matches(&fixture()));
    }

    #[test]
    fn rejects_when_any_fails() {
        let f = Filter::default().with(FilterValue::Collection(vec!["federalist_papers".into()]));
        assert!(!f.matches(&fixture()));
    }
}
