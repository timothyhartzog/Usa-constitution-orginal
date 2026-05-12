//! Burkhard–Keller tree (BK-tree) over the term vocabulary, with
//! Levenshtein-distance metric.
//!
//! ## Why
//!
//! When a query term has zero hits in the inverted index — typically because
//! the user mistyped it — we want to find the *nearest* indexed terms within
//! a small edit distance, expand the query with them, and re-run BM25.
//!
//! ## Construction
//!
//! Built lazily by [`crate::Archive`] from the inverted-index vocabulary on
//! load, so the binary archive format does not need to change.
//!
//! ## Cost
//!
//! For a vocabulary of ~22k terms with average length ≤ 8 the build runs in
//! a few tens of milliseconds; queries with `max_distance = 2` typically
//! visit a few hundred nodes.

use std::cmp::min;

/// In-memory BK-tree. Public for use by [`crate::Archive`]; not exposed
/// directly through the JS surface.
#[derive(Debug, Clone)]
pub struct BkTree {
    nodes: Vec<Node>,
}

#[derive(Debug, Clone)]
struct Node {
    term: String,
    /// `(distance_to_parent, child_index)` pairs.
    children: Vec<(u32, usize)>,
}

impl BkTree {
    /// Builds a tree from a term iterator. Duplicate terms are folded.
    pub fn from_terms<I, S>(terms: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut tree = Self { nodes: Vec::new() };
        for t in terms {
            tree.insert(t.into());
        }
        tree
    }

    /// Number of distinct terms.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` when no terms have been inserted.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    fn insert(&mut self, term: String) {
        if self.nodes.is_empty() {
            self.nodes.push(Node {
                term,
                children: Vec::new(),
            });
            return;
        }
        let mut current = 0usize;
        loop {
            let d = levenshtein(&self.nodes[current].term, &term);
            if d == 0 {
                return; // duplicate
            }
            // Borrow the children slot fresh each iteration to keep the borrow
            // checker happy when descending.
            let mut next: Option<usize> = None;
            for (cd, idx) in &self.nodes[current].children {
                if *cd == d {
                    next = Some(*idx);
                    break;
                }
            }
            match next {
                Some(child) => current = child,
                None => {
                    let new_idx = self.nodes.len();
                    self.nodes.push(Node {
                        term,
                        children: Vec::new(),
                    });
                    self.nodes[current].children.push((d, new_idx));
                    return;
                }
            }
        }
    }

    /// Returns up to `max_results` indexed terms within `max_distance`
    /// of `query`, sorted by ascending edit distance and then lexicographically.
    pub fn search(&self, query: &str, max_distance: u32, max_results: usize) -> Vec<(String, u32)> {
        if self.nodes.is_empty() || max_results == 0 {
            return Vec::new();
        }
        let mut hits: Vec<(String, u32)> = Vec::new();
        let mut stack: Vec<usize> = vec![0];
        while let Some(idx) = stack.pop() {
            let node = &self.nodes[idx];
            let d = levenshtein(&node.term, query);
            if d <= max_distance {
                hits.push((node.term.clone(), d));
            }
            let lo = d.saturating_sub(max_distance);
            let hi = d.saturating_add(max_distance);
            for (cd, child) in &node.children {
                if *cd >= lo && *cd <= hi {
                    stack.push(*child);
                }
            }
        }
        hits.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        hits.truncate(max_results);
        hits
    }
}

/// Standard two-row Levenshtein distance.
///
/// Returns the number of single-character insertions, deletions, or
/// substitutions required to transform `a` into `b`. Operates on Unicode
/// scalar values (`char`s), not bytes.
pub fn levenshtein(a: &str, b: &str) -> u32 {
    if a == b {
        return 0;
    }
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();
    if m == 0 {
        return n as u32;
    }
    if n == 0 {
        return m as u32;
    }

    let mut prev: Vec<u32> = (0..=n as u32).collect();
    let mut curr: Vec<u32> = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i as u32;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = min(min(curr[j - 1] + 1, prev[j] + 1), prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_basic() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("flaw", "lawn"), 2);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("a", ""), 1);
        assert_eq!(levenshtein("", "abc"), 3);
    }

    #[test]
    fn levenshtein_unicode() {
        // Each "é" is one scalar; substitution cost = 1.
        assert_eq!(levenshtein("café", "cafe"), 1);
    }

    #[test]
    fn bk_tree_finds_typos() {
        let tree = BkTree::from_terms([
            "constitution",
            "convention",
            "ratification",
            "federalism",
            "federalist",
        ]);
        let hits = tree.search("constituton", 2, 5); // missing 'i'
        assert!(hits.iter().any(|(t, _)| t == "constitution"));
        // Distance is exactly 1.
        let (_, d) = hits.iter().find(|(t, _)| t == "constitution").unwrap();
        assert_eq!(*d, 1);
    }

    #[test]
    fn bk_tree_returns_sorted_results() {
        let tree = BkTree::from_terms(["congress", "concord", "concert"]);
        let hits = tree.search("conress", 2, 10);
        // closest first
        assert_eq!(hits[0].0, "congress"); // distance 1
    }

    #[test]
    fn bk_tree_respects_max_distance() {
        let tree = BkTree::from_terms(["alpha", "omega"]);
        let hits = tree.search("zzzzz", 1, 10);
        assert!(hits.is_empty());
    }

    #[test]
    fn bk_tree_drops_duplicates() {
        let tree = BkTree::from_terms(["foo", "foo", "foo"]);
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn bk_tree_empty_query_handled() {
        let tree = BkTree::from_terms(["a", "ab", "abc"]);
        let hits = tree.search("", 2, 10);
        // "a" → dist 1, "ab" → dist 2, "abc" → dist 3 (excluded)
        assert_eq!(hits.len(), 2);
    }
}
