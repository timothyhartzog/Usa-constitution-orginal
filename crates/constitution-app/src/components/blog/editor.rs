use dioxus::prelude::*;
use pulldown_cmark::{html, Parser};

use crate::state::{use_blog, BlogPost};

#[component]
pub fn BlogEditorPage() -> Element {
    let mut blog_state = use_blog();

    let mut title = use_signal(|| blog_state.read().draft_title.clone());
    let mut markdown = use_signal(|| blog_state.read().draft_markdown.clone());
    let mut tags_input = use_signal(String::new);
    let mut preview_html = use_signal(String::new);

    let mut update_preview = move |md: &str| {
        let parser = Parser::new(md);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        preview_html.set(html_output);
    };

    let save_draft = move |_| {
        let mut state = blog_state.write();
        state.draft_title = title.read().clone();
        state.draft_markdown = markdown.read().clone();
    };

    let publish = move |_| {
        let t = title.read().clone();
        let md = markdown.read().clone();
        let tags_str = tags_input.read().clone();

        if t.trim().is_empty() || md.trim().is_empty() {
            return;
        }

        let slug = t
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();

        let parser = Parser::new(&md);
        let mut html_out = String::new();
        html::push_html(&mut html_out, parser);

        let excerpt: String = md.chars().take(200).collect();
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let post = BlogPost {
            slug,
            title: t,
            date: "2026-05-27".to_string(),
            tags,
            excerpt,
            html: html_out,
        };

        let mut state = blog_state.write();
        state.posts.push(post);
        state.draft_title.clear();
        state.draft_markdown.clear();
    };

    rsx! {
        div { class: "page blog-editor-page",
            header { class: "page-header",
                div { class: "page-header-row",
                    div {
                        h2 { "Blog Editor" }
                        p { class: "page-subtitle", "Write in Markdown with live preview." }
                    }
                    div { class: "editor-actions",
                        button {
                            class: "btn btn-secondary",
                            onclick: save_draft,
                            "Save Draft"
                        }
                        button {
                            class: "btn btn-primary",
                            onclick: publish,
                            "Publish"
                        }
                    }
                }
            }
            div { class: "editor-meta",
                input {
                    class: "editor-title-input",
                    r#type: "text",
                    placeholder: "Post title...",
                    value: "{title}",
                    oninput: move |e| title.set(e.value()),
                }
                input {
                    class: "editor-tags-input",
                    r#type: "text",
                    placeholder: "Tags (comma-separated)...",
                    value: "{tags_input}",
                    oninput: move |e| tags_input.set(e.value()),
                }
            }
            div { class: "editor-split",
                div { class: "editor-pane",
                    h4 { "Markdown" }
                    textarea {
                        class: "editor-textarea",
                        placeholder: "Write your post in Markdown...\n\nYou can embed widgets: {{{{widget:search query=\"due process\"}}}}",
                        value: "{markdown}",
                        oninput: move |e| {
                            let val = e.value();
                            markdown.set(val.clone());
                            update_preview(&val);
                        },
                    }
                }
                div { class: "preview-pane",
                    h4 { "Preview" }
                    div { class: "preview-content",
                        dangerous_inner_html: "{preview_html}"
                    }
                }
            }
        }
    }
}
