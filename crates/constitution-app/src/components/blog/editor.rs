use dioxus::prelude::*;
use pulldown_cmark::{html, Parser};

use crate::components::blog::widget_render::render_post_blocks;
use crate::router::Route;
use crate::state::{use_blog, BlogPost, Theme};
use crate::storage;

const CURRENT_DATE: &str = "2026-05-28";

#[component]
pub fn BlogEditorPage() -> Element {
    let mut blog_state = use_blog();
    let navigator = use_navigator();

    let initial_title = blog_state.read().draft.title.clone();
    let initial_markdown = blog_state.read().draft.markdown.clone();
    let initial_tags = blog_state.read().draft.tags.clone();

    let mut title = use_signal(|| initial_title);
    let mut markdown = use_signal(|| initial_markdown);
    let mut tags_input = use_signal(|| initial_tags);
    let mut saved_status = use_signal(|| Option::<String>::None);

    let save_draft = move |_| {
        let draft = crate::state::BlogDraft {
            title: title.read().clone(),
            markdown: markdown.read().clone(),
            tags: tags_input.read().clone(),
        };
        blog_state.write().draft = draft.clone();
        if let Ok(json) = serde_json::to_string(&draft) {
            storage::set(storage::KEY_DRAFT, &json);
        }
        saved_status.set(Some("Draft saved".to_string()));
    };

    let clear_draft = move |_| {
        title.set(String::new());
        markdown.set(String::new());
        tags_input.set(String::new());
        blog_state.write().draft = crate::state::BlogDraft::default();
        storage::remove(storage::KEY_DRAFT);
        saved_status.set(Some("Draft cleared".to_string()));
    };

    let publish = move |_| {
        let t = title.read().clone();
        let md = markdown.read().clone();
        let tags_str = tags_input.read().clone();

        if t.trim().is_empty() || md.trim().is_empty() {
            saved_status.set(Some("Title and body are required".to_string()));
            return;
        }

        let slug = slugify(&t);

        let parser = Parser::new(&md);
        let mut html_out = String::new();
        html::push_html(&mut html_out, parser);

        let excerpt: String = md
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#') && !l.starts_with("{{"))
            .next()
            .map(|l| l.chars().take(220).collect())
            .unwrap_or_default();

        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let post = BlogPost {
            slug: slug.clone(),
            title: t,
            date: CURRENT_DATE.to_string(),
            tags,
            excerpt,
            html: html_out,
            markdown: md,
            user_created: true,
        };

        // Insert / replace the post at the top of the list
        {
            let mut state = blog_state.write();
            state.posts.retain(|p| p.slug != post.slug);
            state.posts.insert(0, post.clone());
            state.draft = crate::state::BlogDraft::default();
        }

        // Persist user posts to localStorage
        let user_posts: Vec<BlogPost> = blog_state
            .read()
            .posts
            .iter()
            .filter(|p| p.user_created)
            .cloned()
            .collect();
        if let Ok(json) = serde_json::to_string(&user_posts) {
            storage::set(storage::KEY_POSTS, &json);
        }
        storage::remove(storage::KEY_DRAFT);

        navigator.push(Route::BlogPostPage { slug });
    };

    let preview_md = markdown.read().clone();
    let preview_html = render_markdown_to_html(&preview_md);
    let preview_blocks = render_post_blocks(&preview_html);
    let status_text = saved_status.read().clone();

    rsx! {
        div { class: "page blog-editor-page",
            header { class: "page-header",
                div { class: "page-header-row",
                    div {
                        h2 { "Blog Editor" }
                        p { class: "page-subtitle",
                            "Write in Markdown. Embed widgets with {{{{widget:name args}}}}. Drafts autosave to your browser."
                        }
                    }
                    div { class: "editor-actions",
                        if let Some(status) = status_text {
                            span { class: "editor-status", "{status}" }
                        }
                        button {
                            class: "btn btn-ghost",
                            onclick: clear_draft,
                            "Clear"
                        }
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
                    oninput: move |e| {
                        title.set(e.value());
                        saved_status.set(None);
                    },
                }
                input {
                    class: "editor-tags-input",
                    r#type: "text",
                    placeholder: "Tags (comma-separated)...",
                    value: "{tags_input}",
                    oninput: move |e| {
                        tags_input.set(e.value());
                        saved_status.set(None);
                    },
                }
            }
            div { class: "editor-help",
                "Widgets you can embed: "
                code { "{{widget:search query=\"due process\"}}" }
                ", "
                code { "{{widget:stats}}" }
                ", "
                code { "{{widget:mini_graph target=\"clause:I.8\"}}" }
                ", "
                code { "{{widget:compare topic=\"executive power\"}}" }
            }
            div { class: "editor-split",
                div { class: "editor-pane",
                    h4 { "Markdown" }
                    textarea {
                        class: "editor-textarea",
                        placeholder: "# Your post\n\nWrite in Markdown.\n\nEmbed live widgets:\n\n{{widget:search query=\"due process\"}}",
                        value: "{markdown}",
                        oninput: move |e| {
                            markdown.set(e.value());
                            saved_status.set(None);
                        },
                    }
                }
                div { class: "preview-pane",
                    h4 { "Live Preview" }
                    div { class: "preview-content blog-post-page post-content",
                        if preview_md.trim().is_empty() {
                            p { class: "preview-empty",
                                "Start typing to see a live preview. Widgets render as live components."
                            }
                        } else {
                            for (i, block) in preview_blocks.iter().enumerate() {
                                {
                                    let key = format!("preview-{i}");
                                    match block {
                                        crate::components::blog::widget_render::PostBlock::Html(s) => rsx! {
                                            div {
                                                key: "{key}",
                                                dangerous_inner_html: "{s}",
                                            }
                                        },
                                        crate::components::blog::widget_render::PostBlock::Widget(spec) => {
                                            let name = spec.name.clone();
                                            let args = spec.args.clone();
                                            rsx! {
                                                div { key: "{key}",
                                                    { crate::components::blog::widget_render::render_widget(&name, &args) }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn render_markdown_to_html(md: &str) -> String {
    let parser = Parser::new(md);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    html_out
}

fn slugify(title: &str) -> String {
    let mut out = String::new();
    let mut last_dash = true;
    for c in title.chars() {
        if c.is_alphanumeric() {
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

// Keep theme symbol used as a re-export safety check; not used in this file.
#[allow(dead_code)]
fn _theme_marker(_: Theme) {}
