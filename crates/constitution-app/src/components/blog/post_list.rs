use dioxus::prelude::*;

use crate::router::Route;
use crate::state::use_blog;

#[component]
pub fn BlogIndexPage() -> Element {
    let blog_state = use_blog();
    let state = blog_state.read();

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
            if state.posts.is_empty() {
                div { class: "empty-state",
                    h3 { "No posts yet" }
                    p { "Create your first blog post to get started." }
                    Link {
                        to: Route::BlogEditorPage {},
                        class: "btn btn-primary",
                        "Write a Post"
                    }
                }
            } else {
                div { class: "blog-list",
                    for post in state.posts.iter() {
                        article { class: "blog-card",
                            Link {
                                to: Route::BlogPostPage { slug: post.slug.clone() },
                                h3 { class: "blog-card-title", "{post.title}" }
                            }
                            div { class: "blog-card-meta",
                                span { class: "blog-date", "{post.date}" }
                                for tag in post.tags.iter() {
                                    span { class: "blog-tag", "{tag}" }
                                }
                            }
                            p { class: "blog-excerpt", "{post.excerpt}" }
                        }
                    }
                }
            }
        }
    }
}
