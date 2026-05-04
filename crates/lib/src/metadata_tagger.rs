//! Metadata tagging using keyword matching against constitutional taxonomy
//!
//! Implements keyword-based matching to tag chunks with
//! constitutional clauses and issue categories.

use crate::error::Result;
use crate::types::Chunk;
use crate::tokenizer::normalize_text;
use std::collections::HashSet;

/// Constitutional clause definition
#[derive(Debug, Clone)]
pub struct ConstitutionalClause {
    pub id: String,
    pub display_name: String,
    pub keywords: Vec<String>,
}

/// Issue tag definition
#[derive(Debug, Clone)]
pub struct IssueTag {
    pub id: String,
    pub display_name: String,
    pub keywords: Vec<String>,
}

/// Metadata tagger using keyword matching
#[derive(Debug, Clone)]
pub struct MetadataTagger {
    clauses: Vec<ConstitutionalClause>,
    issue_tags: Vec<IssueTag>,
}

impl MetadataTagger {
    /// Create a new metadata tagger
    pub fn new(
        clauses: Vec<ConstitutionalClause>,
        issue_tags: Vec<IssueTag>,
    ) -> Self {
        Self {
            clauses,
            issue_tags,
        }
    }

    /// Tag a chunk with constitutional clauses and issue tags
    pub fn tag_chunk(&self, chunk: &mut Chunk) -> Result<()> {
        let (issue_tags, clause_tags) = self.tag_text(&chunk.text)?;
        chunk.issue_tags = issue_tags;
        chunk.clause_tags = clause_tags;
        Ok(())
    }

    /// Tag chunk text and return (issue_tags, clause_tags)
    pub fn tag_text(&self, text: &str) -> Result<(Vec<String>, Vec<String>)> {
        let normalized = normalize_text(text).to_lowercase();
        let mut matched_clauses = Vec::new();
        let mut matched_issues = Vec::new();

        // Match constitutional clauses
        for clause in &self.clauses {
            if self.keyword_matches(&normalized, &clause.keywords) {
                matched_clauses.push(clause.id.clone());
            }
        }

        // Match issue tags
        for issue in &self.issue_tags {
            if self.keyword_matches(&normalized, &issue.keywords) {
                matched_issues.push(issue.id.clone());
            }
        }

        // Deduplicate while preserving order
        let clause_tags = dedup_preserve_order(matched_clauses);
        let issue_tags = dedup_preserve_order(matched_issues);

        Ok((issue_tags, clause_tags))
    }

    /// Check if any keyword matches in the text
    fn keyword_matches(&self, text: &str, keywords: &[String]) -> bool {
        keywords.iter().any(|keyword| {
            let kw_lower = keyword.to_lowercase();
            // Match whole phrases or words
            text.contains(&kw_lower)
        })
    }
}

/// Deduplicate a vector while preserving insertion order
fn dedup_preserve_order(items: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tagger_creation() {
        let tagger = MetadataTagger::new(vec![], vec![]);
        assert!(tagger.clauses.is_empty());
        assert!(tagger.issue_tags.is_empty());
    }

    #[test]
    fn test_tag_text_no_matches() {
        let tagger = MetadataTagger::new(vec![], vec![]);
        let (issues, clauses) = tagger.tag_text("random text about nothing").unwrap();
        assert!(issues.is_empty());
        assert!(clauses.is_empty());
    }

    #[test]
    fn test_tag_text_clause_match() {
        let clause = ConstitutionalClause {
            id: "I.1".to_string(),
            display_name: "Legislative Power".to_string(),
            keywords: vec!["legislative".to_string(), "congress".to_string()],
        };

        let tagger = MetadataTagger::new(vec![clause], vec![]);
        let (issues, clauses) = tagger
            .tag_text("All legislative powers herein granted shall be vested in Congress")
            .unwrap();

        assert!(issues.is_empty());
        assert_eq!(clauses.len(), 1);
        assert_eq!(clauses[0], "I.1");
    }

    #[test]
    fn test_tag_text_issue_match() {
        let issue = IssueTag {
            id: "federalism".to_string(),
            display_name: "Federalism".to_string(),
            keywords: vec!["federal".to_string(), "state".to_string()],
        };

        let tagger = MetadataTagger::new(vec![], vec![issue]);
        let (issues, clauses) = tagger
            .tag_text("The federal government and the states share power")
            .unwrap();

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0], "federalism");
        assert!(clauses.is_empty());
    }

    #[test]
    fn test_tag_text_multiple_matches() {
        let clause1 = ConstitutionalClause {
            id: "I.1".to_string(),
            display_name: "Legislative Power".to_string(),
            keywords: vec!["legislative".to_string()],
        };
        let clause2 = ConstitutionalClause {
            id: "II.1".to_string(),
            display_name: "Executive Power".to_string(),
            keywords: vec!["executive".to_string()],
        };

        let tagger = MetadataTagger::new(vec![clause1, clause2], vec![]);
        let (_, clauses) = tagger
            .tag_text("The legislative and executive branches have different powers")
            .unwrap();

        assert_eq!(clauses.len(), 2);
        assert!(clauses.contains(&"I.1".to_string()));
        assert!(clauses.contains(&"II.1".to_string()));
    }

    #[test]
    fn test_tag_text_deduplication() {
        let clause = ConstitutionalClause {
            id: "I.1".to_string(),
            display_name: "Legislative Power".to_string(),
            keywords: vec!["legislative".to_string(), "congress".to_string()],
        };

        let tagger = MetadataTagger::new(vec![clause], vec![]);
        let (_, clauses) = tagger
            .tag_text("Congress and the legislative branch decide laws")
            .unwrap();

        // Should only appear once despite matching multiple keywords
        assert_eq!(clauses.len(), 1);
        assert_eq!(clauses[0], "I.1");
    }

    #[test]
    fn test_tag_chunk() {
        use crate::types::{ChunkId, DocumentId};
        use std::collections::HashMap;

        let clause = ConstitutionalClause {
            id: "I.1".to_string(),
            display_name: "Legislative Power".to_string(),
            keywords: vec!["legislative".to_string()],
        };

        let tagger = MetadataTagger::new(vec![clause], vec![]);
        let mut chunk = Chunk {
            id: ChunkId::new("doc1_0000"),
            document_id: DocumentId::new("doc1"),
            title: "Test".to_string(),
            text: "The legislative branch makes laws".to_string(),
            word_count: 5,
            preview: "The legislative branch...".to_string(),
            issue_tags: vec![],
            clause_tags: vec![],
            metadata: HashMap::new(),
        };

        tagger.tag_chunk(&mut chunk).unwrap();
        assert_eq!(chunk.clause_tags.len(), 1);
        assert_eq!(chunk.clause_tags[0], "I.1");
    }
}
