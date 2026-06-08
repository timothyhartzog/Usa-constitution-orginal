use dioxus::prelude::*;

use crate::components::blog::widget_render::{render_post_blocks, render_widget, PostBlock};
use crate::router::Route;
use crate::state::use_blog;

#[component]
pub fn BlogPostPage(slug: String) -> Element {
    let blog_state = use_blog();
    let state = blog_state.read();

    let post = state.posts.iter().find(|p| p.slug == slug);

    match post {
        Some(post) => {
            let blocks = render_post_blocks(&post.html);
            rsx! {
                div { class: "page blog-post-page",
                    header { class: "post-header",
                        Link { to: Route::BlogIndexPage {}, class: "back-link", "<- All posts" }
                        h1 { class: "post-title", "{post.title}" }
                        div { class: "post-meta",
                            span { class: "post-date", "{post.date}" }
                            for tag in post.tags.iter() {
                                span { class: "post-tag", "{tag}" }
                            }
                            if post.user_created {
                                span { class: "post-badge", "Your post" }
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
        }
        None => rsx! {
            div { class: "page",
                h2 { "Post not found" }
                p { "The blog post \"{slug}\" was not found." }
                Link { to: Route::BlogIndexPage {}, "Back to Blog" }
            }
        },
    }
}
