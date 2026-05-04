//! Text tokenization and normalization
//!
//! Provides language-aware tokenization with stopword filtering,
//! Unicode normalization, and configurable token length constraints.

use crate::error::Result;
use regex::Regex;
use std::collections::HashSet;
use unicode_normalization::UnicodeNormalization;

lazy_static::lazy_static! {
    /// Default English stopwords (58 words)
    static ref DEFAULT_STOPWORDS: HashSet<&'static str> = {
        vec![
            "a", "an", "and", "are", "as", "at", "be", "by", "for", "from",
            "has", "he", "in", "is", "it", "its", "of", "on", "or", "that",
            "the", "to", "was", "will", "with", "would", "can", "could",
            "have", "having", "i", "me", "my", "we", "you", "your", "this",
            "these", "those", "which", "who", "what", "when", "where", "why",
            "how", "all", "each", "every", "both", "few", "more", "most",
            "some", "such", "no", "nor", "not", "only", "own", "so", "than",
            "too", "very", "do", "does", "did", "then", "just",
        ]
        .into_iter()
        .collect()
    };

    /// Regex for tokenization (alphanumeric + apostrophes)
    static ref TOKEN_REGEX: Regex = Regex::new(r"[a-z0-9']+").unwrap();
}

/// Text tokenizer with configurable options
#[derive(Debug, Clone)]
pub struct Tokenizer {
    stopwords: HashSet<String>,
    min_length: usize,
    max_length: usize,
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer {
    /// Create a new tokenizer with default settings
    pub fn new() -> Self {
        Self {
            stopwords: DEFAULT_STOPWORDS.iter().map(|s| s.to_string()).collect(),
            min_length: 3,
            max_length: 50,
        }
    }

    /// Create a tokenizer with custom stopwords
    pub fn with_stopwords(stopwords: Vec<String>) -> Self {
        let mut tokenizer = Self::new();
        tokenizer.stopwords = stopwords.into_iter().collect();
        tokenizer
    }

    /// Set minimum token length
    pub fn with_min_length(mut self, min_length: usize) -> Self {
        self.min_length = min_length;
        self
    }

    /// Set maximum token length
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = max_length;
        self
    }

    /// Tokenize text into lowercase tokens, filtering stopwords and length constraints
    pub fn tokenize(&self, text: &str) -> Result<Vec<String>> {
        let normalized = normalize_text(text);
        let tokens: Vec<String> = TOKEN_REGEX
            .find_iter(&normalized)
            .map(|m| m.as_str().to_string())
            .filter(|token| {
                token.len() >= self.min_length
                    && token.len() <= self.max_length
                    && !self.stopwords.contains(token)
            })
            .collect();

        Ok(tokens)
    }

    /// Tokenize and preserve order while removing duplicates
    pub fn tokenize_unique(&self, text: &str) -> Result<Vec<String>> {
        let tokens = self.tokenize(text)?;
        let mut seen = HashSet::new();
        Ok(tokens
            .into_iter()
            .filter(|token| seen.insert(token.clone()))
            .collect())
    }

    /// Split text into sentences (simple heuristic)
    pub fn split_sentences(&self, text: &str) -> Vec<String> {
        text.split(|c| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Split text into paragraphs
    pub fn split_paragraphs(&self, text: &str) -> Vec<String> {
        text.split("\n\n")
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect()
    }
}

/// Normalize text: NFKC form, lowercase, whitespace collapse
pub fn normalize_text(text: &str) -> String {
    let normalized: String = text.nfkc().collect();
    let lowercased = normalized.to_lowercase();

    // Collapse multiple spaces, tabs, newlines
    let mut result = String::new();
    let mut prev_space = false;

    for c in lowercased.chars() {
        if c.is_whitespace() {
            if !prev_space {
                result.push(' ');
                prev_space = true;
            }
        } else {
            result.push(c);
            prev_space = false;
        }
    }

    result.trim().to_string()
}

/// Create URL-safe slug from text
pub fn slugify(text: &str) -> String {
    let normalized = normalize_text(text);
    normalized
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else if c.is_whitespace() {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("The Constitution was ratified in 1788.").unwrap();
        assert!(tokens.contains(&"constitution".to_string()));
        assert!(tokens.contains(&"ratified".to_string()));
        assert!(!tokens.contains(&"the".to_string())); // stopword
    }

    #[test]
    fn test_tokenize_unique() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer
            .tokenize_unique("The Constitution Constitution Constitution")
            .unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], "constitution");
    }

    #[test]
    fn test_normalize_text() {
        let text = "  The   QUICK\n\nbrown  fox  ";
        let normalized = normalize_text(text);
        assert_eq!(normalized, "the quick brown fox");
    }

    #[test]
    fn test_slugify() {
        let slug = slugify("The Federalist Papers 1787-1788");
        assert_eq!(slug, "the-federalist-papers-1787-1788");
    }

    #[test]
    fn test_split_sentences() {
        let tokenizer = Tokenizer::new();
        let text = "This is sentence one. This is sentence two! And sentence three?";
        let sentences = tokenizer.split_sentences(text);
        assert_eq!(sentences.len(), 3);
    }

    #[test]
    fn test_split_paragraphs() {
        let tokenizer = Tokenizer::new();
        let text = "Para 1\n\nPara 2\n\nPara 3";
        let paragraphs = tokenizer.split_paragraphs(text);
        assert_eq!(paragraphs.len(), 3);
    }

    #[test]
    fn test_min_length_filter() {
        let tokenizer = Tokenizer::new().with_min_length(5);
        let tokens = tokenizer.tokenize("a bb ccc dddd eeeee").unwrap();
        assert!(tokens.iter().all(|t| t.len() >= 5));
    }

    #[test]
    fn test_unicode_normalization() {
        let tokenizer = Tokenizer::new();
        let text = "café naïve"; // with combining diacritics
        let tokens = tokenizer.tokenize(text).unwrap();
        // Should handle Unicode properly
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_stopwords_filtering() {
        let tokenizer = Tokenizer::new();
        let text = "the a an and or is was be";
        let tokens = tokenizer.tokenize(text).unwrap();
        assert!(tokens.is_empty()); // all stopwords
    }
}
