//! URL fragment <-> app state codec.
//!
//! Format: `#sel=clause:I.8&q=due+process&col=federalist_papers,bill_of_rights&issue=federalism&date=1787`
//!
//! Only state that the user would meaningfully want to share is encoded:
//! the current selection (graph / map / reader cross-view focus) and the
//! search query plus its filters. Routes themselves are in the path, not
//! the fragment, so deep links look like `/search#q=due+process`.

use std::collections::BTreeMap;

use constitution_archive::{Filter, FilterValue};

use crate::state::{SelectionKind, SelectionState};

/// Snapshot of the shareable slice of app state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShareState {
    pub selection: Option<SelectionKind>,
    pub query: String,
    pub collections: Vec<String>,
    pub issues: Vec<String>,
    pub authors: Vec<String>,
    pub date_prefix: Option<String>,
    pub doc_types: Vec<String>,
    pub clauses: Vec<String>,
}

impl ShareState {
    pub fn from_selection_and_search(selection: &SelectionState, query: &str, filter: &Filter) -> Self {
        let mut out = Self {
            selection: selection_to_share(&selection.kind),
            query: query.to_string(),
            ..Default::default()
        };
        for clause in &filter.clauses {
            match clause {
                FilterValue::Collection(v) => out.collections = v.clone(),
                FilterValue::IssueTag(v) => out.issues = v.clone(),
                FilterValue::Author(v) => out.authors = v.clone(),
                FilterValue::DocumentType(v) => out.doc_types = v.clone(),
                FilterValue::ClauseTag(v) => out.clauses = v.clone(),
                FilterValue::DatePrefix(s) => out.date_prefix = Some(s.clone()),
            }
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        self.selection.is_none()
            && self.query.is_empty()
            && self.collections.is_empty()
            && self.issues.is_empty()
            && self.authors.is_empty()
            && self.date_prefix.is_none()
            && self.doc_types.is_empty()
            && self.clauses.is_empty()
    }
}

fn selection_to_share(kind: &SelectionKind) -> Option<SelectionKind> {
    // A chunk selection is implicit from the route (we don't want every
    // page visit to add chunk:xxx to the URL); other selections are worth
    // sharing.
    match kind {
        SelectionKind::None | SelectionKind::Chunk(_) => None,
        other => Some(other.clone()),
    }
}

/// Serialize `ShareState` into a URL fragment (without the leading `#`).
/// Returns an empty string when nothing meaningful is set.
pub fn encode(state: &ShareState) -> String {
    if state.is_empty() {
        return String::new();
    }
    let mut params: BTreeMap<&'static str, String> = BTreeMap::new();
    if let Some(sel) = &state.selection {
        params.insert("sel", encode_selection(sel));
    }
    if !state.query.is_empty() {
        params.insert("q", encode_value(&state.query));
    }
    if !state.collections.is_empty() {
        params.insert("col", encode_list(&state.collections));
    }
    if !state.issues.is_empty() {
        params.insert("issue", encode_list(&state.issues));
    }
    if !state.authors.is_empty() {
        params.insert("author", encode_list(&state.authors));
    }
    if !state.doc_types.is_empty() {
        params.insert("doctype", encode_list(&state.doc_types));
    }
    if !state.clauses.is_empty() {
        params.insert("clause", encode_list(&state.clauses));
    }
    if let Some(d) = &state.date_prefix {
        params.insert("date", encode_value(d));
    }
    params
        .into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&")
}

/// Parse a URL fragment (with or without the leading `#`) into a state.
pub fn decode(fragment: &str) -> ShareState {
    let mut out = ShareState::default();
    let trimmed = fragment.trim_start_matches('#');
    if trimmed.is_empty() {
        return out;
    }
    for pair in trimmed.split('&') {
        let Some((key, value)) = pair.split_once('=') else { continue };
        let decoded = decode_value(value);
        match key {
            "sel" => out.selection = decode_selection(&decoded),
            "q" => out.query = decoded,
            "col" => out.collections = split_list(&decoded),
            "issue" => out.issues = split_list(&decoded),
            "author" => out.authors = split_list(&decoded),
            "doctype" => out.doc_types = split_list(&decoded),
            "clause" => out.clauses = split_list(&decoded),
            "date" => out.date_prefix = if decoded.is_empty() { None } else { Some(decoded) },
            _ => {}
        }
    }
    out
}

fn encode_selection(kind: &SelectionKind) -> String {
    let raw = match kind {
        SelectionKind::None => return String::new(),
        SelectionKind::Clause(k) => format!("clause:{k}"),
        SelectionKind::Person(k) => format!("person:{k}"),
        SelectionKind::Essay(k) => format!("essay:{k}"),
        SelectionKind::Country(k) => format!("country:{k}"),
        SelectionKind::Chunk(k) => format!("chunk:{k}"),
    };
    encode_value(&raw)
}

fn decode_selection(s: &str) -> Option<SelectionKind> {
    let (kind, rest) = s.split_once(':')?;
    Some(match kind {
        "clause" => SelectionKind::Clause(rest.to_string()),
        "person" => SelectionKind::Person(rest.to_string()),
        "essay" => SelectionKind::Essay(rest.to_string()),
        "country" => SelectionKind::Country(rest.to_string()),
        "chunk" => SelectionKind::Chunk(rest.to_string()),
        _ => return None,
    })
}

fn encode_list(values: &[String]) -> String {
    values
        .iter()
        .map(|s| encode_value(s))
        .collect::<Vec<_>>()
        .join(",")
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Minimal RFC-3986-ish percent encoder. We avoid the full url crate
/// because we only need to escape characters that would collide with our
/// fragment grammar: `&`, `=`, `,`, `#`, `?`, plus literal `%` and
/// non-ASCII. Spaces become `+` (form-style) since this is a fragment,
/// not a path; both `+` and `%20` decode back to a space.
fn encode_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ' ' => out.push('+'),
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' | ':' => out.push(c),
            _ => {
                let mut buf = [0u8; 4];
                for b in c.encode_utf8(&mut buf).as_bytes() {
                    out.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    out
}

fn decode_value(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'+' {
            out.push(b' ');
            i += 1;
        } else if b == b'%' && i + 2 < bytes.len() {
            let hi = hex(bytes[i + 1]);
            let lo = hex(bytes[i + 2]);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h << 4) | l);
                i += 3;
            } else {
                out.push(b);
                i += 1;
            }
        } else {
            out.push(b);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(state: ShareState) {
        let encoded = encode(&state);
        let decoded = decode(&encoded);
        assert_eq!(state, decoded, "roundtrip failed for {state:?}");
    }

    #[test]
    fn empty_state() {
        assert_eq!(encode(&ShareState::default()), "");
        assert_eq!(decode(""), ShareState::default());
    }

    #[test]
    fn clause_selection_only() {
        roundtrip(ShareState {
            selection: Some(SelectionKind::Clause("I.8".into())),
            ..Default::default()
        });
    }

    #[test]
    fn search_with_filters() {
        roundtrip(ShareState {
            query: "due process".into(),
            collections: vec!["federalist_papers".into(), "bill_of_rights".into()],
            issues: vec!["individual_rights".into()],
            date_prefix: Some("1789".into()),
            ..Default::default()
        });
    }

    #[test]
    fn punctuation_in_query() {
        roundtrip(ShareState {
            query: "100% & complete".into(),
            ..Default::default()
        });
    }

    #[test]
    fn ignores_chunk_selections() {
        let st = ShareState::from_selection_and_search(
            &SelectionState { kind: SelectionKind::Chunk("foo".into()) },
            "",
            &Filter::default(),
        );
        assert!(st.is_empty());
    }

    #[test]
    fn captures_clause_selection() {
        let st = ShareState::from_selection_and_search(
            &SelectionState { kind: SelectionKind::Clause("Amend.1".into()) },
            "",
            &Filter::default(),
        );
        assert_eq!(st.selection, Some(SelectionKind::Clause("Amend.1".into())));
    }

    #[test]
    fn decode_tolerates_unknown_keys() {
        let st = decode("sel=clause:V&zz=nope&q=hello");
        assert_eq!(st.selection, Some(SelectionKind::Clause("V".into())));
        assert_eq!(st.query, "hello");
    }

    #[test]
    fn decode_handles_plus_spaces() {
        let st = decode("q=due+process+clause");
        assert_eq!(st.query, "due process clause");
    }
}
