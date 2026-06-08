use dioxus::prelude::*;

use crate::components::blog::{BlogEditorPage, BlogIndexPage, BlogPostPage};
use crate::components::dashboard::DashboardPage;
use crate::components::graph::GraphPage;
use crate::components::map::WorldPage;
use crate::components::reader::DocumentPage;
use crate::components::search::SearchPage;
use crate::components::timeline::TimelinePage;

#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    DashboardPage {},

    #[route("/search")]
    SearchPage {},

    #[route("/document/:id")]
    DocumentPage { id: String },

    #[route("/graph")]
    GraphPage {},

    #[route("/world")]
    WorldPage {},

    #[route("/timeline")]
    TimelinePage {},

    #[route("/blog")]
    BlogIndexPage {},

    #[route("/blog/editor")]
    BlogEditorPage {},

    #[route("/blog/:slug")]
    BlogPostPage { slug: String },
}
