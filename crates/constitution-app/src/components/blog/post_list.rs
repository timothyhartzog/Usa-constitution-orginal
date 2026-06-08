use dioxus::prelude::*;

use crate::router::Route;
use crate::state::use_blog;

#[component]
pub fn BlogIndexPage() -> Element {
    let mut blog_state = use_blog();
    let mut query = use_signal(String::new);

    let state = blog_state.read();
    let posts = state.posts.clone();
    let active_tag = state.tag_filter.clone();
    let has_draft = !state.draft.title.is_empty() || !state.draft.markdown.is_empty();
    let draft_title = state.draft.title.clone();
    drop(state);

    let all_tags: Vec<(String, usize)> = {
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for post in &posts {
            for t in &post.tags {
                *counts.entry(t.clone()).or_insert(0) += 1;
            }
        }
        let mut v: Vec<_> = counts.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    };

    let q = query.read().to_lowercase();
    let filtered: Vec<_> = posts
        .iter()
        .filter(|p| {
            let tag_ok = active_tag
                .as_ref()
                .map(|t| p.tags.iter().any(|x| x == t))
                .unwrap_or(true);
            let q_ok = q.is_empty()
                || p.title.to_lowercase().contains(&q)
                || p.excerpt.to_lowercase().contains(&q)
                || p.tags.iter().any(|t| t.to_lowercase().contains(&q));
            tag_ok && q_ok
        })
        .collect();

    rsx! {
        div { class: "page blog-page",
            header { class: "page-header",
                div { class: "page-header-row",
                    div {
                        h2 { "Blog" }
                        p { class: "page-subtitle",
                            "Analysis, commentary, and interactive explorations of constitutional history."
                        }
                    }
                    Link {
                        to: Route::BlogEditorPage {},
                        class: "btn btn-primary",
                        "New Post"
                    }
                }
            }

            if has_draft {
                div { class: "draft-banner",
                    div { class: "draft-banner-text",
                        strong { "You have a saved draft" }
                        if !draft_title.is_empty() {
                            span { class: "draft-banner-title", " — \"{draft_title}\"" }
                        }
                    }
                    Link {
                        to: Route::BlogEditorPage {},
                        class: "btn btn-secondary draft-banner-btn",
                        "Resume editing"
                    }
                }
            }

            div { class: "blog-toolbar",
                input {
                    class: "blog-search-input",
                    r#type: "text",
                    placeholder: "Search posts by title, excerpt, or tag...",
                    value: "{query}",
                    oninput: move |e| query.set(e.value()),
                }
            }

            if !all_tags.is_empty() {
                div { class: "tag-filter-bar",
                    span { class: "tag-filter-label", "Filter:" }
                    button {
                        class: if active_tag.is_none() { "tag-pill tag-pill-active" } else { "tag-pill" },
                        onclick: move |_| blog_state.write().tag_filter = None,
                        "All ({posts.len()})"
                    }
                    for (tag, count) in all_tags.iter() {
                        {
                            let t = tag.clone();
                            let is_active = active_tag.as_deref() == Some(tag);
                            rsx! {
                                button {
                                    class: if is_active { "tag-pill tag-pill-active" } else { "tag-pill" },
                                    onclick: move |_| {
                                        let mut bs = blog_state.write();
                                        bs.tag_filter = if is_active { None } else { Some(t.clone()) };
                                    },
                                    "{tag} ({count})"
                                }
                            }
                        }
                    }
                }
            }

            if filtered.is_empty() {
                div { class: "empty-state",
                    h3 { "No matching posts" }
                    p { "Try adjusting your filter or write a new post." }
                    Link {
                        to: Route::BlogEditorPage {},
                        class: "btn btn-primary",
                        "Write a Post"
                    }
                }
            } else {
                div { class: "blog-list",
                    for post in filtered.iter() {
                        article { class: "blog-card",
                            div { class: "blog-card-header",
                                Link {
                                    to: Route::BlogPostPage { slug: post.slug.clone() },
                                    h3 { class: "blog-card-title", "{post.title}" }
                                }
                                if post.user_created {
                                    span { class: "blog-card-badge", "Your post" }
                                }
                            }
                            div { class: "blog-card-meta",
                                span { class: "blog-date", "{post.date}" }
                                for tag in post.tags.iter() {
                                    span { class: "blog-tag", "{tag}" }
                                }
                            }
                            p { class: "blog-excerpt", "{post.excerpt}" }
                            Link {
                                to: Route::BlogPostPage { slug: post.slug.clone() },
                                class: "blog-card-read-more",
                                "Read post ->"
                            }
                        }
                    }
                }
            }
        }
    }
}
