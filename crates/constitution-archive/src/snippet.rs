//! KWIC (keyword-in-context) snippet generation for search results.
//!
//! Given a chunk's full text and the set of matched query terms, returns the
//! window of `window_chars` characters that contains the highest density of
//! matches, together with `(start, end)` highlight ranges measured in
//! **byte offsets into the snippet string** — ready for the JS layer to wrap
//! in `<mark>` tags safely.

use serde::{Deserialize, Serialize};

/// One snippet rendered for the UI.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Snippet {
    /// The contiguous slice of the source text shown to the user.
    pub text: String,
    /// `(byte_start, byte_end)` ranges within `text` covering matched terms.
    pub highlights: Vec<(usize, usize)>,
    /// `true` when the snippet was truncated at the start.
    pub leading_ellipsis: bool,
    /// `true` when the snippet was truncated at the end.
    pub trailing_ellipsis: bool,
}

impl Snippet {
    /// Empty placeholder — used when no terms match (no preview available).
    pub fn empty() -> Self {
        Self::default()
    }

    /// `true` when the snippet carries no text.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// Builds a [`Snippet`] from a chunk's text and the matched query terms.
///
/// Strategy: collect every case-insensitive byte-range occurrence of any
/// matched term, walk a sliding window of `window_chars` characters, pick
/// the window covering the most occurrences, and clip cleanly at character
/// boundaries.
pub fn make_snippet(text: &str, matched_terms: &[String], window_chars: usize) -> Snippet {
    if text.is_empty() || matched_terms.is_empty() || window_chars == 0 {
        return Snippet::empty();
    }
    let occurrences = find_occurrences(text, matched_terms);
    if occurrences.is_empty() {
        return Snippet::empty();
    }

    // Convert window_chars to a byte budget: ASCII-dominated text means this
    // is usually equal, but we walk char boundaries below.
    let total_chars = text.chars().count();
    let win = window_chars.min(total_chars);

    // Char-index of every char in the source text → byte offset.
    let char_to_byte: Vec<usize> = text
        .char_indices()
        .map(|(b, _)| b)
        .chain(std::iter::once(text.len()))
        .collect();

    // Convert occurrence byte ranges to char indices for window selection.
    let occ_char: Vec<(usize, usize)> = occurrences
        .iter()
        .map(|(bs, be)| {
            (
                byte_to_char(&char_to_byte, *bs),
                byte_to_char(&char_to_byte, *be),
            )
        })
        .collect();

    // Slide a window of `win` chars; pick the one starting at the lowest
    // start char that maximises occurrences fully contained inside it.
    let mut best_start_char = 0usize;
    let mut best_count = 0usize;
    let last_start = total_chars.saturating_sub(win);
    for &(occ_start, _) in &occ_char {
        let s = occ_start.saturating_sub(window_chars / 4).min(last_start);
        let e = s + win;
        let count = occ_char.iter().filter(|(a, b)| *a >= s && *b <= e).count();
        if count > best_count {
            best_count = count;
            best_start_char = s;
        }
    }
    if best_count == 0 {
        // Fall back to the first occurrence centred in the window.
        best_start_char = occ_char[0]
            .0
            .saturating_sub(window_chars / 4)
            .min(last_start);
    }

    let start_char = best_start_char;
    let end_char = (start_char + win).min(total_chars);
    let start_byte = char_to_byte[start_char];
    let end_byte = char_to_byte[end_char];
    let snippet_text = text[start_byte..end_byte].to_string();

    let highlights: Vec<(usize, usize)> = occurrences
        .iter()
        .filter_map(|(bs, be)| {
            if *bs >= start_byte && *be <= end_byte {
                Some((bs - start_byte, be - start_byte))
            } else {
                None
            }
        })
        .collect();

    Snippet {
        text: snippet_text,
        highlights,
        leading_ellipsis: start_char > 0,
        trailing_ellipsis: end_char < total_chars,
    }
}

/// Returns sorted, non-overlapping byte ranges where any matched term occurs
/// in `text` (case-insensitive). Word boundaries are required.
fn find_occurrences(text: &str, matched_terms: &[String]) -> Vec<(usize, usize)> {
    let lower = text.to_lowercase();
    let mut hits: Vec<(usize, usize)> = Vec::new();
    for term in matched_terms {
        if term.is_empty() {
            continue;
        }
        let needle = term.to_lowercase();
        let mut start = 0usize;
        while let Some(pos) = lower[start..].find(&needle) {
            let abs = start + pos;
            let end = abs + needle.len();
            if is_word_boundary(&lower, abs, end) {
                hits.push((abs, end));
            }
            start = abs + needle.len().max(1);
        }
    }
    hits.sort_by_key(|(s, _)| *s);
    dedup_overlapping(hits)
}

fn is_word_boundary(text: &str, start: usize, end: usize) -> bool {
    let before = text[..start]
        .chars()
        .next_back()
        .map(char_is_word)
        .unwrap_or(false);
    let after = text[end..]
        .chars()
        .next()
        .map(char_is_word)
        .unwrap_or(false);
    !before && !after
}

fn char_is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn dedup_overlapping(mut ranges: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    ranges.sort_by_key(|(s, _)| *s);
    let mut out: Vec<(usize, usize)> = Vec::with_capacity(ranges.len());
    for r in ranges {
        if let Some(last) = out.last_mut() {
            if r.0 < last.1 {
                last.1 = last.1.max(r.1);
                continue;
            }
        }
        out.push(r);
    }
    out
}

/// Binary-search the char-index → byte-offset table built by the caller.
fn byte_to_char(char_to_byte: &[usize], byte_pos: usize) -> usize {
    match char_to_byte.binary_search(&byte_pos) {
        Ok(i) => i,
        Err(i) => i.saturating_sub(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_highlights_around_match() {
        let text = "Congress shall make no law respecting an establishment of religion.";
        let s = make_snippet(text, &["religion".into()], 200);
        assert!(s.text.contains("religion"));
        assert_eq!(s.highlights.len(), 1);
        let (a, b) = s.highlights[0];
        assert_eq!(&s.text[a..b], "religion");
    }

    #[test]
    fn picks_densest_window() {
        let mut text = String::new();
        text.push_str(&"x ".repeat(80));
        text.push_str("federalism ");
        text.push_str(&"x ".repeat(80));
        text.push_str("federalism federalism ");
        text.push_str(&"x ".repeat(80));
        let s = make_snippet(&text, &["federalism".into()], 60);
        assert!(s.text.contains("federalism"));
        // Both occurrences in the dense block should be highlighted.
        assert_eq!(s.highlights.len(), 2);
    }

    #[test]
    fn no_terms_returns_empty() {
        let s = make_snippet("anything", &[], 100);
        assert!(s.text.is_empty());
        assert!(s.highlights.is_empty());
    }

    #[test]
    fn no_text_returns_empty() {
        let s = make_snippet("", &["foo".into()], 100);
        assert!(s.text.is_empty());
    }

    #[test]
    fn requires_word_boundary() {
        let text = "preconstitutional debate is not constitutional debate";
        let s = make_snippet(text, &["constitutional".into()], 200);
        assert_eq!(
            s.highlights.len(),
            1,
            "should match only standalone occurrence"
        );
    }

    #[test]
    fn handles_unicode_text() {
        let text = "résumé and résumé and unrelated";
        let s = make_snippet(text, &["résumé".into()], 200);
        assert!(s.text.contains("résumé"));
        assert_eq!(s.highlights.len(), 2);
    }

    #[test]
    fn sets_ellipsis_flags() {
        let mut text = String::new();
        text.push_str(&"x ".repeat(200));
        text.push_str("federalism ");
        text.push_str(&"x ".repeat(200));
        let s = make_snippet(&text, &["federalism".into()], 40);
        assert!(s.leading_ellipsis);
        assert!(s.trailing_ellipsis);
    }
}
