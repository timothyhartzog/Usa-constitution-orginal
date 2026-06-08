use dioxus::prelude::*;

use crate::components::reader::{ClauseComparator, MiniGraph, SearchWidget, StatWidget};
use crate::router::Route;
use crate::state::use_blog;

/// One block of a rendered post: either HTML (from Markdown) or a live widget.
#[derive(Debug, Clone)]
enum PostBlock {
    Html(String),
    Widget(WidgetSpec),
}

#[derive(Debug, Clone)]
struct WidgetSpec {
    name: String,
    args: std::collections::HashMap<String, String>,
}

/// Parse blog HTML and split out widget placeholders.
///
/// Widgets are written in source markdown as `{{widget:name arg=value arg=value}}`.
/// `pulldown-cmark` may or may not escape the braces; we accept both.
fn parse_blocks(html: &str) -> Vec<PostBlock> {
    let mut blocks: Vec<PostBlock> = Vec::new();
    let mut cursor = 0usize;
    let bytes = html.as_bytes();

    let needles: &[&[u8]] = &[b"{{widget:", b"{{{{widget:"];

    while cursor < bytes.len() {
        let mut hit: Option<(usize, usize)> = None;
        for &needle in needles {
            if let Some(pos) = find_subslice(&bytes[cursor..], needle) {
                let start = cursor + pos;
                if hit.map(|(p, _)| start < p).unwrap_or(true) {
                    hit = Some((start, needle.len()));
                }
            }
        }

        let Some((start, needle_len)) = hit else {
            blocks.push(PostBlock::Html(html[cursor..].to_string()));
            break;
        };

        if start > cursor {
            blocks.push(PostBlock::Html(html[cursor..start].to_string()));
        }

        // Find closing braces (}} or }}}})
        let after = start + needle_len;
        let close_needles: &[&[u8]] = if needle_len == 9 {
            &[b"}}"]
        } else {
            &[b"}}}}"]
        };
        let mut close_hit: Option<(usize, usize)> = None;
        for &needle in close_needles {
            if let Some(pos) = find_subslice(&bytes[after..], needle) {
                let end = after + pos;
                if close_hit.map(|(p, _)| end < p).unwrap_or(true) {
                    close_hit = Some((end, needle.len()));
                }
            }
        }

        let Some((end, close_len)) = close_hit else {
            blocks.push(PostBlock::Html(html[cursor..].to_string()));
            break;
        };

        let inner = &html[after..end];
        let spec = parse_widget_spec(inner);
        blocks.push(PostBlock::Widget(spec));
        cursor = end + close_len;
    }

    blocks
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    for i in 0..=haystack.len() - needle.len() {
        if &haystack[i..i + needle.len()] == needle {
            return Some(i);
        }
    }
    None
}

fn parse_widget_spec(inner: &str) -> WidgetSpec {
    // Format: name [arg=value arg=value]
    // value may be quoted with "..."
    let trimmed = inner.trim();
    let mut chars = trimmed.chars().peekable();

    // Read name (up to whitespace)
    let mut name = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            break;
        }
        name.push(c);
        chars.next();
    }

    let mut args = std::collections::HashMap::new();
    loop {
        while chars.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
            chars.next();
        }
        if chars.peek().is_none() {
            break;
        }

        // Read key
        let mut key = String::new();
        while let Some(&c) = chars.peek() {
            if c == '=' || c.is_whitespace() {
                break;
            }
            key.push(c);
            chars.next();
        }
        if key.is_empty() {
            break;
        }
        if chars.peek() != Some(&'=') {
            args.insert(key, "true".to_string());
            continue;
        }
        chars.next(); // consume '='

        // Read value
        let mut value = String::new();
        if chars.peek() == Some(&'"') {
            chars.next();
            while let Some(c) = chars.next() {
                if c == '"' {
                    break;
                }
                value.push(c);
            }
        } else {
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    break;
                }
                value.push(c);
                chars.next();
            }
        }
        args.insert(key, value);
    }

    WidgetSpec { name, args }
}

#[component]
pub fn BlogPostPage(slug: String) -> Element {
    let blog_state = use_blog();
    let state = blog_state.read();

    let post = state.posts.iter().find(|p| p.slug == slug);

    match post {
        Some(post) => {
            let blocks = parse_blocks(&post.html);
            rsx! {
                div { class: "page blog-post-page",
                    header { class: "post-header",
                        Link { to: Route::BlogIndexPage {}, class: "back-link", "Back to Blog" }
                        h1 { class: "post-title", "{post.title}" }
                        div { class: "post-meta",
                            span { class: "post-date", "{post.date}" }
                            for tag in post.tags.iter() {
                                span { class: "post-tag", "{tag}" }
                            }
                        }
                    }
                    article { class: "post-content",
                        for (i, block) in blocks.iter().enumerate() {
                            {
                                let key = format!("block-{i}");
                                match block {
                                    PostBlock::Html(s) => rsx! {
                                        div {
                                            key: "{key}",
                                            class: "post-html-block",
                                            dangerous_inner_html: "{s}",
                                        }
                                    },
                                    PostBlock::Widget(spec) => {
                                        let name = spec.name.clone();
                                        let args = spec.args.clone();
                                        rsx! {
                                            div { key: "{key}", class: "post-widget-block",
                                                { render_widget(&name, &args) }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        None => rsx! {
            div { class: "page",
                h2 { "Post not found" }
                p { "The blog post \"{slug}\" was not found." }
                Link { to: Route::BlogIndexPage {}, "Back to Blog" }
            }
        },
    }
}

fn render_widget(name: &str, args: &std::collections::HashMap<String, String>) -> Element {
    match name {
        "search" => {
            let query = args.get("query").cloned().unwrap_or_default();
            let limit = args.get("limit").and_then(|s| s.parse().ok());
            rsx! { SearchWidget { query, limit } }
        }
        "stat" | "stats" => rsx! { StatWidget {} },
        "graph" | "mini_graph" => {
            let target_key = args
                .get("target")
                .or_else(|| args.get("key"))
                .cloned()
                .unwrap_or_else(|| "clause:I.8".to_string());
            let max_links = args.get("links").and_then(|s| s.parse().ok());
            rsx! { MiniGraph { target_key, max_links } }
        }
        "compare" | "comparator" => {
            let topic = args.get("topic").or_else(|| args.get("query")).cloned().unwrap_or_default();
            let collections = args.get("collections").cloned();
            rsx! { ClauseComparator { topic, collections } }
        }
        _ => rsx! {
            div { class: "widget widget-unknown",
                "Unknown widget: {name}"
            }
        },
    }
}
