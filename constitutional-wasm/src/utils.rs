use lazy_static::lazy_static;
use std::collections::HashSet;
use unicode_normalization::UnicodeNormalization;

lazy_static! {
    static ref STOP_WORDS: HashSet<&'static str> = {
        let words = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "up", "about", "into", "through", "during",
            "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
            "do", "does", "did", "will", "would", "could", "should", "may", "might",
            "must", "can", "this", "that", "these", "those", "i", "you", "he", "she",
            "it", "we", "they", "what", "which", "who", "when", "where", "why", "how",
            "all", "each", "every", "both", "few", "more", "most", "other", "some",
            "such", "no", "nor", "not", "only", "own", "same", "so", "than", "too",
            "very", "as", "if", "just", "am", "be", "been", "being", "it", "its",
        ];
        words.into_iter().collect()
    };
}

pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty() && s.len() >= 3 && !STOP_WORDS.contains(s.as_str()))
        .collect()
}

pub fn normalize(text: &str) -> String {
    text.to_lowercase()
        .nfc()
        .collect::<String>()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
}
