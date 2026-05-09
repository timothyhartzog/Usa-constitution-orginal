//! Extract structured citations from chunk text and index them.
//!
//! Three kinds of citations are recognised:
//!
//! - **Constitutional clauses** — `"Article I, Section 8"`, `"Article III"`,
//!   `"Amendment I"`, `"First Amendment"`, … canonicalised to the same
//!   key format used by [`crate::Chunk::constitutional_clause_tags`]
//!   (`"I.8"`, `"III"`, `"Amend.1"`).
//! - **Federalist / Anti-Federalist essays** — `"Federalist No. 10"`,
//!   `"Brutus, No. I"`, etc. canonicalised to `"federalist:10"`,
//!   `"brutus:1"`.
//! - **Persons** — last names of the principal founders, matched only
//!   against a fixed allow-list to keep precision high. Canonicalised to
//!   the lower-case last name (`"madison"`).
//!
//! The extractor is hand-coded (no regex dependency) so the WASM bundle
//! stays small. It walks each chunk's text once per pattern family.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::chunk::Chunk;

/// What a citation points at.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum CitationTarget {
    /// Constitutional clause, e.g. `"I.8"`, `"III"`, `"Amend.1"`.
    Clause(String),
    /// Numbered essay, e.g. `"federalist:10"`, `"brutus:1"`.
    Essay(String),
    /// Lower-case last name of a known founder, e.g. `"madison"`.
    Person(String),
}

impl CitationTarget {
    /// Stable string key used in the index.
    pub fn key(&self) -> String {
        match self {
            Self::Clause(s) => format!("clause:{s}"),
            Self::Essay(s) => format!("essay:{s}"),
            Self::Person(s) => format!("person:{s}"),
        }
    }
}

/// One extracted citation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// The chunk where the citation appears (chunk index in the archive).
    pub from_chunk: u32,
    /// What the citation points at.
    pub target: CitationTarget,
    /// Byte offset into the chunk text where the matched span starts.
    pub byte_offset: usize,
    /// The text that matched (e.g. `"Article I, Section 8"`).
    pub matched_text: String,
}

/// All citations extracted from a corpus, with two indexes.
#[derive(Debug, Clone, Default)]
pub struct CitationGraph {
    citations: Vec<Citation>,
    by_from: HashMap<u32, Vec<usize>>,
    by_target_key: BTreeMap<String, Vec<usize>>,
}

impl CitationGraph {
    /// Builds the graph by scanning every chunk's text.
    pub fn build(chunks: &[Chunk]) -> Self {
        let mut citations: Vec<Citation> = Vec::new();
        for (idx, chunk) in chunks.iter().enumerate() {
            let from_chunk = idx as u32;
            extract_clauses(&chunk.text, from_chunk, &mut citations);
            extract_amendments(&chunk.text, from_chunk, &mut citations);
            extract_essays(&chunk.text, from_chunk, &mut citations);
            extract_persons(&chunk.text, from_chunk, &mut citations);
        }
        let mut by_from: HashMap<u32, Vec<usize>> = HashMap::new();
        let mut by_target_key: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, c) in citations.iter().enumerate() {
            by_from.entry(c.from_chunk).or_default().push(i);
            by_target_key.entry(c.target.key()).or_default().push(i);
        }
        Self {
            citations,
            by_from,
            by_target_key,
        }
    }

    /// Total citation count.
    pub fn len(&self) -> usize {
        self.citations.len()
    }

    /// Returns `true` when no citations have been extracted.
    pub fn is_empty(&self) -> bool {
        self.citations.is_empty()
    }

    /// All citations whose source is `chunk_idx`.
    pub fn out_citations(&self, chunk_idx: u32) -> Vec<&Citation> {
        self.by_from
            .get(&chunk_idx)
            .map(|v| v.iter().map(|i| &self.citations[*i]).collect())
            .unwrap_or_default()
    }

    /// All citations whose canonical key matches `target_key`
    /// (e.g. `"clause:I.8"`, `"essay:federalist:10"`, `"person:madison"`).
    pub fn cited_by(&self, target_key: &str) -> Vec<&Citation> {
        self.by_target_key
            .get(target_key)
            .map(|v| v.iter().map(|i| &self.citations[*i]).collect())
            .unwrap_or_default()
    }

    /// Returns the top `limit` targets (any kind) with the most incoming
    /// citations, sorted by descending count.
    pub fn top_targets(&self, limit: usize) -> Vec<(String, usize)> {
        let mut counts: Vec<(String, usize)> = self
            .by_target_key
            .iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        counts.truncate(limit);
        counts
    }

    /// Returns a co-occurrence view of the top-`top_n` cited targets.
    ///
    /// Edges are undirected; weight is the number of chunks in which both
    /// endpoints are cited. Edges with weight 0 are dropped. Useful for
    /// visualising the citation network as a force-directed graph.
    pub fn graph_view(&self, top_n: usize) -> CitationGraphView {
        // Materialise the top-N target keys (sorted by descending count).
        let mut counts: Vec<(String, usize)> = self
            .by_target_key
            .iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        counts.truncate(top_n);

        let key_to_idx: HashMap<String, usize> = counts
            .iter()
            .enumerate()
            .map(|(i, (k, _))| (k.clone(), i))
            .collect();

        // For each chunk, get the set of top-N target indices it cites,
        // then increment every pair.
        let mut chunk_targets: HashMap<u32, Vec<usize>> = HashMap::new();
        for c in &self.citations {
            let key = c.target.key();
            if let Some(idx) = key_to_idx.get(&key) {
                let entry = chunk_targets.entry(c.from_chunk).or_default();
                if !entry.contains(idx) {
                    entry.push(*idx);
                }
            }
        }

        let mut edge_weights: HashMap<(usize, usize), u32> = HashMap::new();
        for ids in chunk_targets.values() {
            // Cap the work per chunk: a chunk with >50 unique targets is an
            // outlier and would generate a quadratic explosion of edges.
            let bounded = if ids.len() > 50 { &ids[..50] } else { &ids[..] };
            for i in 0..bounded.len() {
                for j in (i + 1)..bounded.len() {
                    let (a, b) = if bounded[i] < bounded[j] {
                        (bounded[i], bounded[j])
                    } else {
                        (bounded[j], bounded[i])
                    };
                    *edge_weights.entry((a, b)).or_insert(0) += 1;
                }
            }
        }

        let nodes: Vec<CitationNode> = counts
            .iter()
            .map(|(key, count)| {
                let (kind, label) = split_target_key(key);
                CitationNode {
                    key: key.clone(),
                    kind: kind.to_string(),
                    label: label.to_string(),
                    citation_count: *count,
                }
            })
            .collect();

        let mut edges: Vec<CitationEdge> = edge_weights
            .into_iter()
            .map(|((a, b), w)| CitationEdge {
                source: nodes[a].key.clone(),
                target: nodes[b].key.clone(),
                weight: w,
            })
            .collect();
        edges.sort_by(|a, b| b.weight.cmp(&a.weight).then_with(|| a.source.cmp(&b.source)));

        CitationGraphView { nodes, edges }
    }
}

/// Split a canonical citation-target key into `(kind, label)`.
///
/// `"person:madison"` → `("person", "madison")`
/// `"essay:federalist:10"` → `("essay", "federalist:10")`
/// `"clause:I.8"` → `("clause", "I.8")`
fn split_target_key(key: &str) -> (&str, &str) {
    match key.split_once(':') {
        Some((kind, rest)) => (kind, rest),
        None => ("unknown", key),
    }
}

/// One node in the [`CitationGraphView`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationNode {
    /// Canonical target key (e.g. `"person:madison"`).
    pub key: String,
    /// Target kind: `"clause"`, `"essay"`, or `"person"`.
    pub kind: String,
    /// Display label (the part after the kind prefix).
    pub label: String,
    /// Number of citations in the corpus to this target.
    pub citation_count: usize,
}

/// One undirected edge in the [`CitationGraphView`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationEdge {
    /// Canonical key of one endpoint.
    pub source: String,
    /// Canonical key of the other endpoint.
    pub target: String,
    /// Number of chunks in which both endpoints are cited.
    pub weight: u32,
}

/// A renderable view of the citation co-occurrence graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationGraphView {
    /// Top-N nodes sorted by descending citation count.
    pub nodes: Vec<CitationNode>,
    /// Edges over those nodes, sorted by descending weight.
    pub edges: Vec<CitationEdge>,
}

// ---------------------------------------------------------------------------
// Pattern extractors
// ---------------------------------------------------------------------------

/// Map a single Roman-numeral character to its value, or `None`.
fn roman_char_value(c: char) -> Option<u32> {
    match c.to_ascii_uppercase() {
        'I' => Some(1),
        'V' => Some(5),
        'X' => Some(10),
        'L' => Some(50),
        'C' => Some(100),
        'D' => Some(500),
        'M' => Some(1000),
        _ => None,
    }
}

const ORDINAL_AMENDMENTS: &[(&str, u32)] = &[
    ("twenty-seventh", 27), ("twenty-sixth", 26), ("twenty-fifth", 25),
    ("twenty-fourth", 24), ("twenty-third", 23), ("twenty-second", 22),
    ("twenty-first", 21), ("twentieth", 20), ("nineteenth", 19),
    ("eighteenth", 18), ("seventeenth", 17), ("sixteenth", 16),
    ("fifteenth", 15), ("fourteenth", 14), ("thirteenth", 13),
    ("twelfth", 12), ("eleventh", 11), ("tenth", 10), ("ninth", 9),
    ("eighth", 8), ("seventh", 7), ("sixth", 6), ("fifth", 5),
    ("fourth", 4), ("third", 3), ("second", 2), ("first", 1),
];

const FOUNDER_LAST_NAMES: &[&str] = &[
    "madison", "hamilton", "jefferson", "adams", "washington", "franklin",
    "jay", "mason", "henry", "sherman", "wilson", "randolph", "morris",
    "dickinson", "pinckney", "ellsworth", "rutledge", "gerry", "lee",
    "monroe", "marshall", "story", "iredell",
];

/// Generic helper: case-insensitive search for `needle` in `text`, returning
/// every byte position where it occurs as a whole word.
fn find_word_positions(text: &str, needle: &str) -> Vec<usize> {
    let lower = text.to_lowercase();
    let needle = needle.to_lowercase();
    let mut hits = Vec::new();
    let mut start = 0usize;
    while let Some(pos) = lower[start..].find(&needle) {
        let abs = start + pos;
        let end = abs + needle.len();
        let before = lower[..abs].chars().next_back().unwrap_or(' ');
        let after = lower[end..].chars().next().unwrap_or(' ');
        if !before.is_alphanumeric() && !after.is_alphanumeric() {
            hits.push(abs);
        }
        start = abs + needle.len().max(1);
    }
    hits
}

/// Parse a Roman numeral with standard subtractive notation
/// (`I`, `IV`, `IX`, `XL`, `XC`, `CD`, `CM`). Returns `None` if any
/// character is non-Roman or the string is empty.
fn parse_roman(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    let mut total: i32 = 0;
    let mut prev: i32 = 0;
    for c in s.chars().rev() {
        let v = roman_char_value(c)? as i32;
        if v < prev {
            total -= v;
        } else {
            total += v;
            prev = v;
        }
    }
    if total > 0 {
        Some(total as u32)
    } else {
        None
    }
}

/// Read a numeric token (Roman or Arabic) starting at `pos`.
/// Returns `(end_byte, parsed_value, original_str)` if a numeral matches.
fn read_numeral(text: &str, pos: usize) -> Option<(usize, u32, String)> {
    let bytes = text.as_bytes();
    let mut end = pos;
    while end < bytes.len() {
        let b = bytes[end];
        if b.is_ascii_alphanumeric() {
            end += 1;
        } else {
            break;
        }
    }
    if end == pos {
        return None;
    }
    let token = &text[pos..end];
    if let Ok(n) = token.parse::<u32>() {
        return Some((end, n, token.to_string()));
    }
    parse_roman(token).map(|n| (end, n, token.to_string()))
}

fn extract_clauses(text: &str, from_chunk: u32, out: &mut Vec<Citation>) {
    for start in find_word_positions(text, "Article") {
        // Skip whitespace after "Article".
        let after = start + "Article".len();
        let mut p = after;
        while p < text.len() && text.as_bytes()[p].is_ascii_whitespace() {
            p += 1;
        }
        let Some((art_end, art_num, art_token)) = read_numeral(text, p) else {
            continue;
        };
        if !(1..=7).contains(&art_num) {
            continue;
        }
        let art_roman = match art_num {
            1 => "I", 2 => "II", 3 => "III", 4 => "IV", 5 => "V", 6 => "VI", 7 => "VII",
            _ => continue,
        };

        // Look for an optional ", Section N" / " Section N" suffix.
        let mut q = art_end;
        while q < text.len() && matches!(text.as_bytes()[q], b' ' | b',' | b'\t') {
            q += 1;
        }
        let section_keyword = "Section";
        let canonical = if text[q..]
            .to_lowercase()
            .starts_with(&section_keyword.to_lowercase())
        {
            let mut r = q + section_keyword.len();
            while r < text.len() && text.as_bytes()[r].is_ascii_whitespace() {
                r += 1;
            }
            if let Some((sec_end, sec_num, _)) = read_numeral(text, r) {
                let key = format!("{art_roman}.{sec_num}");
                let matched = text[start..sec_end].to_string();
                out.push(Citation {
                    from_chunk,
                    target: CitationTarget::Clause(key),
                    byte_offset: start,
                    matched_text: matched,
                });
                continue;
            } else {
                art_roman.to_string()
            }
        } else {
            art_roman.to_string()
        };

        let matched = text[start..art_end].to_string();
        // Filter out "Article 7" inside a date literal like "1787 article 7" —
        // require the token preceding "Article" to NOT be a 4-digit year.
        if matched.split_whitespace().last().map(|s| s == art_token).unwrap_or(false) {
            out.push(Citation {
                from_chunk,
                target: CitationTarget::Clause(canonical),
                byte_offset: start,
                matched_text: matched,
            });
        }
    }
}

fn extract_amendments(text: &str, from_chunk: u32, out: &mut Vec<Citation>) {
    let lower = text.to_lowercase();
    // Pattern A: "First Amendment", "Second Amendment", ...
    for (lit, n) in ORDINAL_AMENDMENTS {
        let needle = format!("{lit} amendment");
        let mut start = 0usize;
        while let Some(pos) = lower[start..].find(&needle) {
            let abs = start + pos;
            let end = abs + needle.len();
            let before = lower[..abs].chars().next_back().unwrap_or(' ');
            let after = lower[end..].chars().next().unwrap_or(' ');
            if !before.is_alphanumeric() && !after.is_alphanumeric() {
                out.push(Citation {
                    from_chunk,
                    target: CitationTarget::Clause(format!("Amend.{n}")),
                    byte_offset: abs,
                    matched_text: text[abs..end].to_string(),
                });
            }
            start = abs + needle.len();
        }
    }
    // Pattern B: "Amendment I", "Amendment X", "Amendment 1".
    for start in find_word_positions(text, "Amendment") {
        let after = start + "Amendment".len();
        let mut p = after;
        while p < text.len() && text.as_bytes()[p].is_ascii_whitespace() {
            p += 1;
        }
        if let Some((end, n, _)) = read_numeral(text, p) {
            if (1..=27).contains(&n) {
                out.push(Citation {
                    from_chunk,
                    target: CitationTarget::Clause(format!("Amend.{n}")),
                    byte_offset: start,
                    matched_text: text[start..end].to_string(),
                });
            }
        }
    }
}

fn extract_essays(text: &str, from_chunk: u32, out: &mut Vec<Citation>) {
    let series: &[(&str, &str)] = &[
        ("Federalist", "federalist"),
        ("Brutus", "brutus"),
        ("Cato", "cato"),
        ("Centinel", "centinel"),
    ];
    for (label, slug) in series {
        for start in find_word_positions(text, label) {
            // Optional comma + " No. " | " Paper " | " Number " + numeral.
            let after = start + label.len();
            let mut p = after;
            while p < text.len() && matches!(text.as_bytes()[p], b' ' | b',' | b'\t' | b'.') {
                p += 1;
            }
            // Try the keywords.
            let keywords = ["no.", "no", "number", "paper", "letter"];
            let mut matched_keyword = false;
            for kw in &keywords {
                if text[p..]
                    .to_lowercase()
                    .starts_with(&kw.to_lowercase())
                {
                    p += kw.len();
                    matched_keyword = true;
                    break;
                }
            }
            if !matched_keyword {
                continue;
            }
            while p < text.len() && (text.as_bytes()[p].is_ascii_whitespace() || text.as_bytes()[p] == b'.') {
                p += 1;
            }
            if let Some((end, n, _)) = read_numeral(text, p) {
                if (1..=85).contains(&n) {
                    out.push(Citation {
                        from_chunk,
                        target: CitationTarget::Essay(format!("{slug}:{n}")),
                        byte_offset: start,
                        matched_text: text[start..end].to_string(),
                    });
                }
            }
        }
    }
}

fn extract_persons(text: &str, from_chunk: u32, out: &mut Vec<Citation>) {
    for name in FOUNDER_LAST_NAMES {
        for start in find_word_positions(text, name) {
            let end = start + name.len();
            out.push(Citation {
                from_chunk,
                target: CitationTarget::Person(name.to_string()),
                byte_offset: start,
                matched_text: text[start..end].to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::Chunk;

    fn chunk(id: &str, text: &str) -> Chunk {
        Chunk {
            chunk_id: id.into(),
            document_id: id.into(),
            title: "T".into(),
            author: "A".into(),
            date: "1787-09-17".into(),
            source_collection: "constitution".into(),
            source_url: "".into(),
            document_type: "".into(),
            issue_tags: vec![],
            constitutional_clause_tags: vec![],
            text: text.into(),
            word_count: 0,
            preview: String::new(),
        }
    }

    #[test]
    fn extracts_article_section_clauses() {
        let chunks = vec![chunk(
            "a",
            "as set forth in Article I, Section 8 and again in Article III the courts...",
        )];
        let g = CitationGraph::build(&chunks);
        let keys: Vec<String> = g
            .out_citations(0)
            .iter()
            .filter_map(|c| match &c.target {
                CitationTarget::Clause(k) => Some(k.clone()),
                _ => None,
            })
            .collect();
        assert!(keys.contains(&"I.8".to_string()));
        assert!(keys.contains(&"III".to_string()));
    }

    #[test]
    fn extracts_amendment_citations() {
        let chunks = vec![chunk(
            "a",
            "the First Amendment forbids…; see also Amendment IV and Amendment 5.",
        )];
        let g = CitationGraph::build(&chunks);
        let keys: Vec<String> = g
            .out_citations(0)
            .iter()
            .filter_map(|c| match &c.target {
                CitationTarget::Clause(k) => Some(k.clone()),
                _ => None,
            })
            .collect();
        assert!(keys.contains(&"Amend.1".to_string()));
        assert!(keys.contains(&"Amend.4".to_string()));
        assert!(keys.contains(&"Amend.5".to_string()));
    }

    #[test]
    fn extracts_federalist_and_brutus() {
        let chunks = vec![chunk(
            "a",
            "as in Federalist No. 10 and again in Federalist No. LI, Brutus, No. I objects…",
        )];
        let g = CitationGraph::build(&chunks);
        let keys: Vec<String> = g
            .out_citations(0)
            .iter()
            .filter_map(|c| match &c.target {
                CitationTarget::Essay(k) => Some(k.clone()),
                _ => None,
            })
            .collect();
        assert!(keys.contains(&"federalist:10".to_string()));
        assert!(keys.contains(&"federalist:51".to_string()));
        assert!(keys.contains(&"brutus:1".to_string()));
    }

    #[test]
    fn extracts_person_citations() {
        let chunks = vec![chunk(
            "a",
            "Madison and Hamilton drafted; Jefferson, in Paris, replied.",
        )];
        let g = CitationGraph::build(&chunks);
        let names: Vec<String> = g
            .out_citations(0)
            .iter()
            .filter_map(|c| match &c.target {
                CitationTarget::Person(n) => Some(n.clone()),
                _ => None,
            })
            .collect();
        assert!(names.contains(&"madison".to_string()));
        assert!(names.contains(&"hamilton".to_string()));
        assert!(names.contains(&"jefferson".to_string()));
    }

    #[test]
    fn cited_by_indexes_targets() {
        let chunks = vec![
            chunk("a", "Federalist No. 10 was about factions."),
            chunk("b", "as Federalist No. 10 argued, factions are…"),
            chunk("c", "Federalist No. 51 went further."),
        ];
        let g = CitationGraph::build(&chunks);
        let cited = g.cited_by("essay:federalist:10");
        assert_eq!(cited.len(), 2);
        let sources: Vec<u32> = cited.iter().map(|c| c.from_chunk).collect();
        assert!(sources.contains(&0));
        assert!(sources.contains(&1));
    }

    #[test]
    fn top_targets_returns_ranked_list() {
        let chunks = vec![
            chunk("a", "Madison Madison Madison Hamilton"),
            chunk("b", "Madison Hamilton"),
            chunk("c", "Jefferson"),
        ];
        let g = CitationGraph::build(&chunks);
        let top = g.top_targets(3);
        assert_eq!(top[0].0, "person:madison");
        assert!(top[0].1 >= top[1].1);
    }

    #[test]
    fn requires_word_boundaries() {
        // "preamble" should not match "amble"; "carticle" shouldn't match "Article".
        let chunks = vec![chunk("a", "carticle and the constitution")];
        let g = CitationGraph::build(&chunks);
        let clause_count = g
            .out_citations(0)
            .iter()
            .filter(|c| matches!(c.target, CitationTarget::Clause(_)))
            .count();
        assert_eq!(clause_count, 0);
    }

    #[test]
    fn rejects_out_of_range_essay_numbers() {
        let chunks = vec![chunk("a", "Federalist No. 999 should not match")];
        let g = CitationGraph::build(&chunks);
        let essay_count = g
            .out_citations(0)
            .iter()
            .filter(|c| matches!(c.target, CitationTarget::Essay(_)))
            .count();
        assert_eq!(essay_count, 0);
    }

    #[test]
    fn graph_view_top_n_and_co_occurrence() {
        let chunks = vec![
            chunk("a", "Madison and Hamilton met"),
            chunk("b", "Madison and Hamilton agreed; Hamilton wrote"),
            chunk("c", "Jefferson alone"),
        ];
        let g = CitationGraph::build(&chunks);
        let view = g.graph_view(10);

        // All three persons are nodes.
        let keys: Vec<&str> = view.nodes.iter().map(|n| n.key.as_str()).collect();
        assert!(keys.contains(&"person:madison"));
        assert!(keys.contains(&"person:hamilton"));
        assert!(keys.contains(&"person:jefferson"));

        // Madison/Hamilton co-occur in chunks a and b → weight 2.
        let mh = view
            .edges
            .iter()
            .find(|e| {
                (e.source == "person:madison" && e.target == "person:hamilton")
                    || (e.source == "person:hamilton" && e.target == "person:madison")
            })
            .expect("madison/hamilton edge missing");
        assert_eq!(mh.weight, 2);

        // No edge from jefferson (only one chunk, alone).
        assert!(!view
            .edges
            .iter()
            .any(|e| e.source.contains("jefferson") || e.target.contains("jefferson")));

        // Sorted by descending count.
        assert!(view.nodes[0].citation_count >= view.nodes[view.nodes.len() - 1].citation_count);

        // Edges sorted by descending weight.
        assert!(view.edges.windows(2).all(|w| w[0].weight >= w[1].weight));

        // Each node has a kind that matches the key prefix.
        for n in &view.nodes {
            assert!(n.key.starts_with(&format!("{}:", n.kind)));
        }
    }

    #[test]
    fn graph_view_respects_top_n_limit() {
        let chunks = vec![chunk(
            "a",
            "Madison Hamilton Jefferson Adams Washington Franklin Mason Henry Wilson Sherman",
        )];
        let g = CitationGraph::build(&chunks);
        let view = g.graph_view(3);
        assert_eq!(view.nodes.len(), 3);
    }

    #[test]
    fn graph_view_empty_for_unrecognised_corpus() {
        let chunks = vec![chunk("a", "this text references nothing recognisable")];
        let g = CitationGraph::build(&chunks);
        let view = g.graph_view(10);
        assert!(view.nodes.is_empty());
        assert!(view.edges.is_empty());
    }
}
