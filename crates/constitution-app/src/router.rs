use dioxus::prelude::*;

use crate::components::blog::{BlogEditorPage, BlogIndexPage, BlogPostPage};
use crate::components::browse::{AuthorPage, AuthorsIndexPage, CollectionPage, CollectionsIndexPage};
use crate::components::dashboard::DashboardPage;
use crate::components::graph::GraphPage;
use crate::components::map::WorldPage;
use crate::components::reader::{ComparePage, DocumentPage};
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

    #[route("/compare/:a/:b")]
    ComparePage { a: String, b: String },

    #[route("/graph")]
    GraphPage {},

    #[route("/world")]
    WorldPage {},

    #[route("/timeline")]
    TimelinePage {},

    #[route("/authors")]
    AuthorsIndexPage {},

    #[route("/author/:slug")]
    AuthorPage { slug: String },

    #[route("/collections")]
    CollectionsIndexPage {},

    #[route("/collection/:slug")]
    CollectionPage { slug: String },

    #[route("/blog")]
    BlogIndexPage {},

    #[route("/blog/editor")]
    BlogEditorPage {},

    #[route("/blog/:slug")]
    BlogPostPage { slug: String },
}
