//! Roll up chunks by author and by collection into dossier structures
//! used by the `/authors`, `/author/:slug`, `/collections`, and
//! `/collection/:slug` views.
//!
//! All aggregation is pure-data over a borrowed `&[Chunk]`. The output
//! is sorted deterministically (descending by count, ties broken
//! alphabetically) so dossier listings are stable across renders.

use std::collections::HashMap;

use constitution_archive::Chunk;

/// A dossier summarizing one author's footprint in the corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorDossier {
    pub name: String,
    pub slug: String,
    pub chunk_count: usize,
    pub document_count: usize,
    pub collections: Vec<(String, usize)>,
    pub top_issues: Vec<(String, usize)>,
    pub top_clauses: Vec<(String, usize)>,
    pub date_range: Option<(String, String)>,
    /// Up to 6 representative chunk ids the UI may render as sample cards.
    pub sample_chunk_ids: Vec<String>,
}

/// A dossier summarizing one collection's footprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionDossier {
    pub name: String,
    pub slug: String,
    pub chunk_count: usize,
    pub document_count: usize,
    pub authors: Vec<(String, usize)>,
    pub top_issues: Vec<(String, usize)>,
    pub date_range: Option<(String, String)>,
    pub sample_chunk_ids: Vec<String>,
}

/// Slugify a free-form name into a URL-safe token. Lowercase, ASCII
/// alphanumerics + dash. Multiple separators collapse; the result has
/// no leading or trailing dash.
pub fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut last_dash = true;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            for lc in c.to_lowercase() {
                out.push(lc);
            }
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// Build per-author dossiers from a chunk slice. Returns dossiers sorted
/// by chunk_count desc.
pub fn build_author_dossiers(chunks: &[Chunk]) -> Vec<AuthorDossier> {
    let mut by_author: HashMap<String, Accum> = HashMap::new();
    for chunk in chunks {
        let author = normalize_author(&chunk.author);
        if author.is_empty() {
            continue;
        }
        let entry = by_author.entry(author).or_default();
        entry.absorb(chunk);
    }

    let mut dossiers: Vec<AuthorDossier> = by_author
        .into_iter()
        .map(|(name, acc)| acc.into_author_dossier(name))
        .collect();
    dossiers.sort_by(|a, b| {
        b.chunk_count
            .cmp(&a.chunk_count)
            .then_with(|| a.name.cmp(&b.name))
    });
    dossiers
}

/// Build per-collection dossiers from a chunk slice. Sorted by
/// chunk_count desc.
pub fn build_collection_dossiers(chunks: &[Chunk]) -> Vec<CollectionDossier> {
    let mut by_collection: HashMap<String, Accum> = HashMap::new();
    for chunk in chunks {
        if chunk.source_collection.is_empty() {
            continue;
        }
        let entry = by_collection
            .entry(chunk.source_collection.clone())
            .or_default();
        entry.absorb(chunk);
    }

    let mut dossiers: Vec<CollectionDossier> = by_collection
        .into_iter()
        .map(|(name, acc)| acc.into_collection_dossier(name))
        .collect();
    dossiers.sort_by(|a, b| {
        b.chunk_count
            .cmp(&a.chunk_count)
            .then_with(|| a.name.cmp(&b.name))
    });
    dossiers
}

/// Find an author dossier by its slug.
pub fn find_author<'a>(dossiers: &'a [AuthorDossier], slug: &str) -> Option<&'a AuthorDossier> {
    dossiers.iter().find(|d| d.slug == slug)
}

/// Find a collection dossier by its slug.
pub fn find_collection<'a>(
    dossiers: &'a [CollectionDossier],
    slug: &str,
) -> Option<&'a CollectionDossier> {
    dossiers.iter().find(|d| d.slug == slug)
}

fn normalize_author(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("unknown") {
        return String::new();
    }
    trimmed.to_string()
}

#[derive(Default)]
struct Accum {
    chunks: usize,
    documents: std::collections::HashSet<String>,
    collections: HashMap<String, usize>,
    authors: HashMap<String, usize>,
    issues: HashMap<String, usize>,
    clauses: HashMap<String, usize>,
    min_date: Option<String>,
    max_date: Option<String>,
    sample_chunk_ids: Vec<(String, u32)>,
}

impl Accum {
    fn absorb(&mut self, chunk: &Chunk) {
        self.chunks += 1;
        self.documents.insert(chunk.document_id.clone());
        if !chunk.source_collection.is_empty() {
            *self
                .collections
                .entry(chunk.source_collection.clone())
                .or_insert(0) += 1;
        }
        let author = normalize_author(&chunk.author);
        if !author.is_empty() {
            *self.authors.entry(author).or_insert(0) += 1;
        }
        for tag in &chunk.issue_tags {
            *self.issues.entry(tag.clone()).or_insert(0) += 1;
        }
        for tag in &chunk.constitutional_clause_tags {
            *self.clauses.entry(tag.clone()).or_insert(0) += 1;
        }
        if !chunk.date.is_empty() {
            match &mut self.min_date {
                Some(cur) if chunk.date < *cur => *cur = chunk.date.clone(),
                None => self.min_date = Some(chunk.date.clone()),
                _ => {}
            }
            match &mut self.max_date {
                Some(cur) if chunk.date > *cur => *cur = chunk.date.clone(),
                None => self.max_date = Some(chunk.date.clone()),
                _ => {}
            }
        }
        // Sample by largest word_count so the cards show meaty passages
        // first, with a cap of 32 candidates held during accumulation.
        self.sample_chunk_ids
            .push((chunk.chunk_id.clone(), chunk.word_count));
        if self.sample_chunk_ids.len() > 32 {
            self.sample_chunk_ids.sort_by(|a, b| b.1.cmp(&a.1));
            self.sample_chunk_ids.truncate(16);
        }
    }

    fn finalize_samples(mut self) -> Vec<String> {
        self.sample_chunk_ids.sort_by(|a, b| b.1.cmp(&a.1));
        self.sample_chunk_ids
            .into_iter()
            .take(6)
            .map(|(id, _)| id)
            .collect()
    }

    fn into_author_dossier(self, name: String) -> AuthorDossier {
        let slug = slugify(&name);
        let chunk_count = self.chunks;
        let document_count = self.documents.len();
        let collections = top_n_sorted(&self.collections, 10);
        let top_issues = top_n_sorted(&self.issues, 8);
        let top_clauses = top_n_sorted(&self.clauses, 8);
        let date_range = pair_dates(&self.min_date, &self.max_date);
        let sample_chunk_ids = self.finalize_samples();
        AuthorDossier {
            name,
            slug,
            chunk_count,
            document_count,
            collections,
            top_issues,
            top_clauses,
            date_range,
            sample_chunk_ids,
        }
    }

    fn into_collection_dossier(self, name: String) -> CollectionDossier {
        let slug = slugify(&name);
        let chunk_count = self.chunks;
        let document_count = self.documents.len();
        let authors = top_n_sorted(&self.authors, 10);
        let top_issues = top_n_sorted(&self.issues, 8);
        let date_range = pair_dates(&self.min_date, &self.max_date);
        let sample_chunk_ids = self.finalize_samples();
        CollectionDossier {
            name,
            slug,
            chunk_count,
            document_count,
            authors,
            top_issues,
            date_range,
            sample_chunk_ids,
        }
    }
}

fn top_n_sorted(map: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

fn pair_dates(min: &Option<String>, max: &Option<String>) -> Option<(String, String)> {
    match (min, max) {
        (Some(a), Some(b)) => Some((a.clone(), b.clone())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(chunks: Vec<(&str, &str, &str, &str, Vec<&str>, u32)>) -> Vec<Chunk> {
        chunks
            .into_iter()
            .map(|(id, author, date, collection, issues, wc)| Chunk {
                chunk_id: id.to_string(),
                document_id: id.split('_').next().unwrap_or(id).to_string(),
                title: format!("Title for {id}"),
                author: author.to_string(),
                date: date.to_string(),
                source_collection: collection.to_string(),
                source_url: String::new(),
                document_type: "doc".to_string(),
                issue_tags: issues.into_iter().map(String::from).collect(),
                constitutional_clause_tags: vec![],
                text: format!("Text body for {id}"),
                word_count: wc,
                preview: String::new(),
            })
            .collect()
    }

    #[test]
    fn slugify_lowercases_and_dashifies() {
        assert_eq!(slugify("James Madison"), "james-madison");
        assert_eq!(slugify("  Federalist  Papers!"), "federalist-papers");
        assert_eq!(slugify("Hamilton, Madison & Jay"), "hamilton-madison-jay");
        assert_eq!(slugify(""), "");
        assert_eq!(slugify("---"), "");
        assert_eq!(slugify("ACL_2024"), "acl-2024");
    }

    #[test]
    fn author_dossiers_count_and_sort() {
        let chunks = fixture(vec![
            (
                "a_1",
                "James Madison",
                "1787",
                "constitution",
                vec!["federalism"],
                100,
            ),
            (
                "b_1",
                "James Madison",
                "1788",
                "federalist_papers",
                vec!["federalism"],
                80,
            ),
            ("c_1", "Brutus", "1788", "anti_federalist", vec![], 60),
            ("d_1", "Unknown", "1789", "constitution", vec![], 90),
            ("e_1", "", "1789", "constitution", vec![], 90),
        ]);
        let d = build_author_dossiers(&chunks);
        // Unknown / empty are dropped, Madison is largest (2 chunks across 2 docs).
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].name, "James Madison");
        assert_eq!(d[0].chunk_count, 2);
        assert_eq!(d[0].document_count, 2);
        assert_eq!(d[0].slug, "james-madison");
        // Two distinct collections.
        assert_eq!(d[0].collections.len(), 2);
        assert_eq!(d[1].name, "Brutus");
    }

    #[test]
    fn author_date_range_is_min_max() {
        let chunks = fixture(vec![
            ("a_1", "Madison", "1789-09-25", "constitution", vec![], 10),
            ("a_2", "Madison", "1788-02-10", "constitution", vec![], 10),
            ("a_3", "Madison", "1791-12-15", "constitution", vec![], 10),
        ]);
        let d = build_author_dossiers(&chunks);
        let (mn, mx) = d[0].date_range.clone().expect("date range");
        assert_eq!(mn, "1788-02-10");
        assert_eq!(mx, "1791-12-15");
    }

    #[test]
    fn collection_dossier_authors_sorted_desc() {
        let chunks = fixture(vec![
            ("a_1", "Madison", "1787", "federalist_papers", vec![], 50),
            ("a_2", "Madison", "1787", "federalist_papers", vec![], 50),
            ("b_1", "Hamilton", "1787", "federalist_papers", vec![], 50),
            ("b_2", "Hamilton", "1787", "federalist_papers", vec![], 50),
            ("b_3", "Hamilton", "1787", "federalist_papers", vec![], 50),
            ("c_1", "Jay", "1787", "federalist_papers", vec![], 50),
        ]);
        let d = build_collection_dossiers(&chunks);
        assert_eq!(d.len(), 1);
        let c = &d[0];
        assert_eq!(c.name, "federalist_papers");
        assert_eq!(c.chunk_count, 6);
        assert_eq!(c.authors[0], ("Hamilton".to_string(), 3));
        assert_eq!(c.authors[1], ("Madison".to_string(), 2));
        assert_eq!(c.authors[2], ("Jay".to_string(), 1));
    }

    #[test]
    fn samples_prefer_meatier_chunks() {
        let chunks = fixture(vec![
            ("a_1", "Madison", "1787", "constitution", vec![], 5),
            ("a_2", "Madison", "1787", "constitution", vec![], 200),
            ("a_3", "Madison", "1787", "constitution", vec![], 800),
            ("a_4", "Madison", "1787", "constitution", vec![], 50),
        ]);
        let d = build_author_dossiers(&chunks);
        // a_3 (800 wc) should come first.
        assert_eq!(d[0].sample_chunk_ids[0], "a_3");
    }

    #[test]
    fn find_by_slug_roundtrips() {
        let chunks = fixture(vec![(
            "a",
            "James Madison",
            "1787",
            "constitution",
            vec![],
            10,
        )]);
        let d = build_author_dossiers(&chunks);
        assert!(find_author(&d, "james-madison").is_some());
        assert!(find_author(&d, "nope").is_none());
    }
}
