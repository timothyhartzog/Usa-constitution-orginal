//! Export helpers: convert structured app data into portable formats
//! (JSON, CSV, Markdown) and trigger a browser download.
//!
//! All functions are pure data transformers in `format::*`; the WASM
//! download trigger is isolated in `download_blob`.

use constitution_archive::{Chunk, SearchHit};

use crate::state::{Annotation, ArchiveState, HistoryEntry};

/// Renders a list of search results as JSON, sized for a portable export.
pub fn search_results_json(query: &str, hits: &[SearchHit], state: &ArchiveState) -> String {
    #[derive(serde::Serialize)]
    struct Out<'a> {
        query: &'a str,
        total: usize,
        results: Vec<HitOut<'a>>,
    }
    #[derive(serde::Serialize)]
    struct HitOut<'a> {
        chunk_id: &'a str,
        score: f32,
        matched_terms: &'a [String],
        snippet: &'a str,
        chunk: Option<ChunkOut>,
    }
    #[derive(serde::Serialize)]
    struct ChunkOut {
        title: String,
        author: String,
        date: String,
        collection: String,
        document_type: String,
        document_id: String,
        word_count: u32,
        preview: String,
    }

    let results: Vec<HitOut> = hits
        .iter()
        .map(|h| {
            let chunk = state.chunk(&h.chunk_id).map(|c| ChunkOut {
                title: c.title,
                author: c.author,
                date: c.date,
                collection: c.source_collection,
                document_type: c.document_type,
                document_id: c.document_id,
                word_count: c.word_count,
                preview: c.preview,
            });
            HitOut {
                chunk_id: &h.chunk_id,
                score: h.score,
                matched_terms: &h.matched_terms,
                snippet: &h.snippet.text,
                chunk,
            }
        })
        .collect();

    serde_json::to_string_pretty(&Out {
        query,
        total: hits.len(),
        results,
    })
    .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
}

/// Search results as RFC 4180 CSV. Snippets and text fields are
/// double-quoted and embedded quotes are doubled.
pub fn search_results_csv(hits: &[SearchHit], state: &ArchiveState) -> String {
    let mut out = String::from(
        "chunk_id,title,author,date,collection,score,matched_terms,snippet\n",
    );
    for hit in hits {
        let chunk = state.chunk(&hit.chunk_id);
        let title = chunk.as_ref().map(|c| c.title.clone()).unwrap_or_default();
        let author = chunk.as_ref().map(|c| c.author.clone()).unwrap_or_default();
        let date = chunk.as_ref().map(|c| c.date.clone()).unwrap_or_default();
        let collection = chunk
            .as_ref()
            .map(|c| c.source_collection.clone())
            .unwrap_or_default();
        let terms = hit.matched_terms.join(" ");
        out.push_str(&csv_field(&hit.chunk_id));
        out.push(',');
        out.push_str(&csv_field(&title));
        out.push(',');
        out.push_str(&csv_field(&author));
        out.push(',');
        out.push_str(&csv_field(&date));
        out.push(',');
        out.push_str(&csv_field(&collection));
        out.push(',');
        out.push_str(&format!("{:.4}", hit.score));
        out.push(',');
        out.push_str(&csv_field(&terms));
        out.push(',');
        out.push_str(&csv_field(&strip_html(&hit.snippet.text)));
        out.push('\n');
    }
    out
}

fn csv_field(s: &str) -> String {
    if s.contains('"') || s.contains(',') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn strip_html(s: &str) -> String {
    // Small inline-tag stripper: removes <mark>...</mark> wrappers added
    // by the snippet builder. Good enough for plain-text export.
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// Search results as Markdown for pasting into a note-taking app.
pub fn search_results_markdown(query: &str, hits: &[SearchHit], state: &ArchiveState) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Search: \"{query}\"\n\n"));
    out.push_str(&format!("_{} results_\n\n", hits.len()));
    for hit in hits {
        let chunk = state.chunk(&hit.chunk_id);
        let title = chunk
            .as_ref()
            .map(|c| c.title.as_str())
            .unwrap_or("Unknown");
        out.push_str(&format!("## {title}\n\n"));
        if let Some(ref c) = chunk {
            if !c.author.is_empty() || !c.date.is_empty() {
                out.push_str(&format!(
                    "*{author}{sep}{date}*\n\n",
                    author = c.author,
                    sep = if !c.author.is_empty() && !c.date.is_empty() { " · " } else { "" },
                    date = c.date,
                ));
            }
            out.push_str(&format!(
                "Collection: `{}` · Score: {:.2}\n\n",
                c.source_collection, hit.score
            ));
        }
        let snippet = strip_html(&hit.snippet.text);
        if !snippet.is_empty() {
            out.push_str(&format!("> {}\n\n", snippet.replace('\n', "\n> ")));
        }
        out.push_str(&format!("ID: `{}`\n\n---\n\n", hit.chunk_id));
    }
    out
}

/// Single chunk as Markdown with frontmatter, suitable for archiving
/// or re-importing.
pub fn chunk_markdown(chunk: &Chunk) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("title: {}\n", chunk.title));
    out.push_str(&format!("author: {}\n", chunk.author));
    out.push_str(&format!("date: {}\n", chunk.date));
    out.push_str(&format!("collection: {}\n", chunk.source_collection));
    out.push_str(&format!("document_id: {}\n", chunk.document_id));
    out.push_str(&format!("chunk_id: {}\n", chunk.chunk_id));
    if !chunk.source_url.is_empty() {
        out.push_str(&format!("source_url: {}\n", chunk.source_url));
    }
    if !chunk.issue_tags.is_empty() {
        out.push_str(&format!("issues: [{}]\n", chunk.issue_tags.join(", ")));
    }
    if !chunk.constitutional_clause_tags.is_empty() {
        out.push_str(&format!(
            "clauses: [{}]\n",
            chunk.constitutional_clause_tags.join(", ")
        ));
    }
    out.push_str("---\n\n");
    out.push_str(&format!("# {}\n\n", chunk.title));
    out.push_str(&chunk.text);
    out.push('\n');
    out
}

/// BibTeX entry for a single chunk. Type defaults to `@misc` since the
/// platform's chunks aren't a precise match for any standard type;
/// authoritative collections (constitution / bill_of_rights) become
/// `@book` and Federalist/Anti-Federalist essays become `@article`.
pub fn chunk_bibtex(chunk: &Chunk) -> String {
    let key = citation_key(chunk);
    let entry_type = match chunk.source_collection.as_str() {
        "constitution" | "bill_of_rights" => "book",
        "federalist_papers" | "anti_federalist" => "article",
        "founders_correspondence" | "letters_delegates_congress" => "misc",
        "madisons_notes" | "elliots_debates" => "book",
        "comparative_constitutions_world" | "comparative_constitutions_eu" => "book",
        _ => "misc",
    };

    let mut out = String::new();
    out.push_str(&format!("@{entry_type}{{{key},\n"));
    out.push_str(&format!("  title = {{{}}},\n", bib_escape(&chunk.title)));
    if !chunk.author.is_empty() {
        out.push_str(&format!("  author = {{{}}},\n", bib_escape(&chunk.author)));
    }
    if !chunk.date.is_empty() {
        if let Some(year) = extract_year(&chunk.date) {
            out.push_str(&format!("  year = {{{year}}},\n"));
        }
        out.push_str(&format!("  date = {{{}}},\n", bib_escape(&chunk.date)));
    }
    out.push_str(&format!(
        "  note = {{Chunk {} in collection \"{}\"}},\n",
        bib_escape(&chunk.chunk_id),
        bib_escape(&chunk.source_collection)
    ));
    if !chunk.source_url.is_empty() {
        out.push_str(&format!("  url = {{{}}},\n", chunk.source_url));
    }
    out.push_str("}\n");
    out
}

/// BibTeX for many chunks.
#[allow(dead_code)]
pub fn chunks_bibtex(chunks: &[Chunk]) -> String {
    chunks.iter().map(chunk_bibtex).collect::<Vec<_>>().join("\n")
}

/// RIS (Research Information Systems) entry for a single chunk. RIS is
/// the import format used by Zotero / Mendeley / EndNote.
pub fn chunk_ris(chunk: &Chunk) -> String {
    let ty = match chunk.source_collection.as_str() {
        "constitution" | "bill_of_rights" | "madisons_notes" | "elliots_debates" => "BOOK",
        "federalist_papers" | "anti_federalist" => "JOUR",
        "founders_correspondence" | "letters_delegates_congress" => "MANSCPT",
        "comparative_constitutions_world" | "comparative_constitutions_eu" => "BOOK",
        _ => "GEN",
    };

    let mut out = String::new();
    out.push_str(&format!("TY  - {ty}\n"));
    out.push_str(&format!("TI  - {}\n", chunk.title));
    if !chunk.author.is_empty() {
        for author in split_authors(&chunk.author) {
            out.push_str(&format!("AU  - {author}\n"));
        }
    }
    if let Some(year) = extract_year(&chunk.date) {
        out.push_str(&format!("PY  - {year}\n"));
    }
    if !chunk.date.is_empty() {
        out.push_str(&format!("DA  - {}\n", chunk.date));
    }
    if !chunk.source_url.is_empty() {
        out.push_str(&format!("UR  - {}\n", chunk.source_url));
    }
    out.push_str(&format!(
        "N1  - Chunk {} in collection \"{}\"\n",
        chunk.chunk_id, chunk.source_collection
    ));
    out.push_str("ER  - \n");
    out
}

/// RIS for many chunks.
#[allow(dead_code)]
pub fn chunks_ris(chunks: &[Chunk]) -> String {
    chunks.iter().map(chunk_ris).collect::<Vec<_>>().join("\n")
}

/// Plain-text formatted reference (Chicago-style). Useful for inline
/// pasting into prose.
pub fn chunk_citation_plain(chunk: &Chunk) -> String {
    let mut out = String::new();
    if !chunk.author.is_empty() {
        out.push_str(&chunk.author);
        out.push_str(". ");
    }
    out.push('"');
    out.push_str(&chunk.title);
    out.push('"');
    if !chunk.date.is_empty() {
        out.push_str(", ");
        out.push_str(&chunk.date);
    }
    if !chunk.source_collection.is_empty() {
        out.push_str(&format!(" ({}).", chunk.source_collection.replace('_', " ")));
    } else {
        out.push('.');
    }
    if !chunk.source_url.is_empty() {
        out.push(' ');
        out.push_str(&chunk.source_url);
        out.push('.');
    }
    out
}

/// BibTeX-safe citation key built from author + year + first significant
/// word from the title. Only [a-zA-Z0-9_:-] survive.
pub fn citation_key(chunk: &Chunk) -> String {
    let author = chunk
        .author
        .split_whitespace()
        .last()
        .map(|s| s.trim_end_matches(','))
        .unwrap_or("anon")
        .to_lowercase();
    let year = extract_year(&chunk.date).unwrap_or_else(|| "nd".into());
    let word = chunk
        .title
        .split_whitespace()
        .find(|w| {
            let l = w.to_lowercase();
            !matches!(
                l.as_str(),
                "the" | "a" | "an" | "of" | "on" | "in" | "to" | "for" | "and"
            )
        })
        .unwrap_or("doc")
        .to_lowercase();
    sanitize_key(&format!("{}-{}-{}-{}", author, year, word, short_chunk_id(&chunk.chunk_id)))
}

fn sanitize_key(s: &str) -> String {
    s.chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':') {
                Some(c)
            } else if c.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect()
}

fn short_chunk_id(chunk_id: &str) -> &str {
    // Strip the document_id prefix to keep keys readable.
    chunk_id.rsplit('_').next().unwrap_or(chunk_id)
}

fn bib_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('{', "\\{")
        .replace('}', "\\}")
}

fn extract_year(date: &str) -> Option<String> {
    for token in date.split(|c: char| !c.is_ascii_digit()) {
        if token.len() == 4 {
            if let Ok(n) = token.parse::<u32>() {
                if (1500..2200).contains(&n) {
                    return Some(token.to_string());
                }
            }
        }
    }
    None
}

fn split_authors(author: &str) -> Vec<String> {
    // "Hamilton, Madison, Jay" -> ["Hamilton", "Madison", "Jay"]
    // "James Madison" -> ["James Madison"]
    // "Madison & Hamilton" -> ["Madison", "Hamilton"]
    let mut out: Vec<String> = Vec::new();
    for part in author.split(|c: char| c == ',' || c == '&' || c == ';') {
        let p = part.trim();
        if !p.is_empty() {
            out.push(p.to_string());
        }
    }
    if out.is_empty() {
        out.push(author.to_string());
    }
    out
}

/// Bookmarks + annotations as JSON.
pub fn library_json(bookmarks: &[HistoryEntry], annotations: &[Annotation]) -> String {
    #[derive(serde::Serialize)]
    struct Out<'a> {
        bookmarks: &'a [HistoryEntry],
        annotations: &'a [Annotation],
    }
    serde_json::to_string_pretty(&Out {
        bookmarks,
        annotations,
    })
    .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
}

/// Trigger a download in the browser. No-op on native.
pub fn download(filename: &str, mime: &str, contents: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        download_browser(filename, mime, contents)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (filename, mime, contents);
        Err("download() is only available in the browser".into())
    }
}

#[cfg(target_arch = "wasm32")]
fn download_browser(filename: &str, mime: &str, contents: &str) -> Result<(), String> {
    use js_sys::Array;
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;

    let parts = Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(contents));

    let bag = BlobPropertyBag::new();
    bag.set_type(mime);

    let blob = Blob::new_with_str_sequence_and_options(&parts, &bag)
        .map_err(|e| format!("Blob construct failed: {e:?}"))?;
    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("URL.createObjectURL failed: {e:?}"))?;

    let anchor: HtmlAnchorElement = document
        .create_element("a")
        .map_err(|e| format!("create_element failed: {e:?}"))?
        .dyn_into()
        .map_err(|_| "anchor cast failed".to_string())?;
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor
        .set_attribute("style", "display:none")
        .ok();
    document
        .body()
        .ok_or("no body")?
        .append_child(&anchor)
        .map_err(|e| format!("append_child failed: {e:?}"))?;
    anchor.click();
    anchor.remove();

    // Schedule revocation; if it fails it's a one-off browser leak.
    let _ = Url::revoke_object_url(&url);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Chunk {
        Chunk {
            chunk_id: "us_constitution_1787_article_i_section_8_0000".into(),
            document_id: "us_constitution_1787".into(),
            title: "Constitution of the United States".into(),
            author: "Constitutional Convention".into(),
            date: "1787-09-17".into(),
            source_collection: "constitution".into(),
            source_url: "https://example.org/c".into(),
            document_type: "foundational_document".into(),
            issue_tags: vec!["federalism".into()],
            constitutional_clause_tags: vec!["I.8".into()],
            text: "We the People...".into(),
            word_count: 4,
            preview: "We the People...".into(),
        }
    }

    #[test]
    fn extract_year_picks_first_plausible() {
        assert_eq!(extract_year("1787-09-17"), Some("1787".into()));
        assert_eq!(extract_year("September 17, 1787"), Some("1787".into()));
        assert_eq!(extract_year("17/09/1787"), Some("1787".into()));
        assert_eq!(extract_year("Year 999 wasn't a year here"), None);
        assert_eq!(extract_year("c. 1789–1791"), Some("1789".into()));
        assert_eq!(extract_year("no date"), None);
    }

    #[test]
    fn citation_key_uses_author_year_word() {
        let key = citation_key(&fixture());
        assert!(key.starts_with("convention-1787-"), "{key}");
        assert!(key.contains("constitution"), "{key}");
    }

    #[test]
    fn bibtex_emits_book_for_constitution() {
        let s = chunk_bibtex(&fixture());
        assert!(s.starts_with("@book{"), "{s}");
        assert!(s.contains("title = {Constitution of the United States}"));
        assert!(s.contains("year = {1787}"));
        assert!(s.contains("url = {https://example.org/c}"));
    }

    #[test]
    fn bibtex_escapes_braces_in_titles() {
        let mut c = fixture();
        c.title = "Some {weird} title".into();
        let s = chunk_bibtex(&c);
        assert!(s.contains("title = {Some \\{weird\\} title}"), "{s}");
    }

    #[test]
    fn ris_lists_each_author_separately() {
        let mut c = fixture();
        c.author = "Hamilton, Madison, Jay".into();
        let s = chunk_ris(&c);
        let au_lines: Vec<_> = s.lines().filter(|l| l.starts_with("AU  - ")).collect();
        assert_eq!(au_lines.len(), 3, "{s}");
    }

    #[test]
    fn plain_citation_is_human_readable() {
        let s = chunk_citation_plain(&fixture());
        assert!(s.contains("Constitutional Convention"));
        assert!(s.contains("1787-09-17"));
        assert!(s.contains("constitution"));
    }

    #[test]
    fn split_authors_handles_separators() {
        assert_eq!(split_authors("Madison"), vec!["Madison"]);
        assert_eq!(split_authors("Madison, Hamilton"), vec!["Madison", "Hamilton"]);
        assert_eq!(split_authors("Madison & Hamilton"), vec!["Madison", "Hamilton"]);
    }
}
