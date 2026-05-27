use dioxus::prelude::*;

use crate::router::Route;
use crate::state::use_blog;

#[component]
pub fn BlogPostPage(slug: String) -> Element {
    let blog_state = use_blog();
    let state = blog_state.read();

    let post = state.posts.iter().find(|p| p.slug == slug);

    match post {
        Some(post) => rsx! {
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
                    dangerous_inner_html: "{post.html}"
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
