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
