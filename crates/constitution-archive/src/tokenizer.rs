//! Lightweight tokenizer used by the inverted index.
//!
//! Strategy:
//!  - NFKC-normalize input
//!  - lowercase
//!  - split on any non-alphanumeric character
//!  - drop tokens shorter than `MIN_TOKEN_LEN`
//!  - drop a small fixed stop-word list
//!
//! Deterministic across native and `wasm32` builds.

use std::collections::HashSet;

use unicode_normalization::UnicodeNormalization;

/// Minimum token length kept in the inverted index.
pub const MIN_TOKEN_LEN: usize = 2;

/// Returns the canonical English stop-word set used by the index.
pub fn stopwords() -> HashSet<&'static str> {
    [
        "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "from", "has", "have", "he",
        "her", "him", "his", "i", "if", "in", "into", "is", "it", "its", "of", "on", "or", "our",
        "she", "such", "than", "that", "the", "their", "them", "then", "there", "these", "they",
        "this", "those", "to", "us", "was", "we", "were", "what", "when", "which", "who", "will",
        "with", "would", "you", "your", "shall", "may", "upon",
    ]
    .into_iter()
    .collect()
}

/// Tokenizes `text` for indexing or query parsing.
///
/// Returns lowercase tokens with stop-words removed but **preserves order
/// and duplicates** so callers can compute term frequencies.
pub fn tokenize(text: &str) -> Vec<String> {
    let stop = stopwords();
    text.nfkc()
        .collect::<String>()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() >= MIN_TOKEN_LEN && !stop.contains(w.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_punctuation_and_case() {
        let toks = tokenize("The People, in Order to form a more perfect Union…");
        assert_eq!(toks, vec!["people", "order", "form", "more", "perfect", "union"]);
    }

    #[test]
    fn unicode_normalized() {
        // U+00E9 (é) and "e\u{0301}" should produce the same token stream.
        let a = tokenize("résumé");
        let b = tokenize("re\u{0301}sume\u{0301}");
        assert_eq!(a, b);
    }

    #[test]
    fn drops_short_and_stop() {
        let toks = tokenize("a I am the quick brown fox");
        assert!(!toks.contains(&"a".to_string()));
        assert!(!toks.contains(&"the".to_string()));
        assert!(toks.contains(&"quick".to_string()));
    }
}
