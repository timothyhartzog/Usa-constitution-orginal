#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

mod components;
mod router;
mod state;
mod storage;

use std::rc::Rc;

use dioxus::prelude::*;

use components::nav::Sidebar;
use router::Route;
use state::{
    ArchiveState, BlogDraft, BlogPost, BlogState, SearchState, SelectionState, Theme,
    WorldConstitutionMeta,
};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(ArchiveState {
        loading: true,
        ..Default::default()
    }));
    use_context_provider(|| Signal::new(SelectionState::default()));
    use_context_provider(|| Signal::new(SearchState::default()));
    use_context_provider(|| Signal::new(load_initial_blog_state()));
    use_context_provider(|| Signal::new(load_initial_theme()));

    let mut archive_state = state::use_archive();

    use_future(move || async move {
        match load_archive_data().await {
            Ok((archive, world_meta)) => {
                let mut state = archive_state.write();
                state.archive = Some(Rc::new(archive));
                state.world_meta = world_meta;
                state.loading = false;
                state.error = None;
            }
            Err(e) => {
                let mut state = archive_state.write();
                state.loading = false;
                state.error = Some(e);
            }
        }
    });

    let theme = state::use_theme();
    let theme_class = match *theme.read() {
        Theme::System => "theme-system",
        Theme::Light => "theme-light",
        Theme::Dark => "theme-dark",
    };

    rsx! {
        document::Stylesheet { href: asset!("/assets/main.css") }
        main { class: "app-shell {theme_class}",
            Sidebar {}
            section { class: "app-content",
                Router::<Route> {}
            }
        }
    }
}

fn load_initial_blog_state() -> BlogState {
    // Built-in posts compiled from content/blog/*.md
    let mut posts = built_in_posts();

    // Layer on user-published posts from localStorage
    if let Some(raw) = storage::get(storage::KEY_POSTS) {
        if let Ok(user_posts) = serde_json::from_str::<Vec<BlogPost>>(&raw) {
            for p in user_posts {
                posts.push(p);
            }
        }
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date));

    // Load any saved draft
    let draft = storage::get(storage::KEY_DRAFT)
        .and_then(|raw| serde_json::from_str::<BlogDraft>(&raw).ok())
        .unwrap_or_default();

    BlogState {
        posts,
        draft,
        tag_filter: None,
    }
}

fn load_initial_theme() -> Theme {
    storage::get(storage::KEY_THEME)
        .map(|s| Theme::from_str(&s))
        .unwrap_or_default()
}

async fn load_archive_data() -> Result<(constitution_archive::Archive, Vec<WorldConstitutionMeta>), String> {
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;

        let archive_bytes = Request::get("/assets/constitution_archive.bin")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch archive: {e}"))?
            .binary()
            .await
            .map_err(|e| format!("Failed to read archive bytes: {e}"))?;

        let archive = constitution_archive::Archive::load(&archive_bytes)
            .map_err(|e| format!("Failed to parse archive: {e}"))?;

        let world_meta: Vec<WorldConstitutionMeta> = match Request::get("/assets/world_meta.json")
            .send()
            .await
        {
            Ok(resp) => resp
                .json()
                .await
                .unwrap_or_default(),
            Err(_) => Vec::new(),
        };

        Ok((archive, world_meta))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let archive_path = std::path::Path::new("data/index/constitution_archive.bin");
        if !archive_path.exists() {
            return Err("Archive not found. Run `cargo run --bin build-archive` first.".into());
        }
        let bytes = std::fs::read(archive_path)
            .map_err(|e| format!("Failed to read archive: {e}"))?;
        let archive = constitution_archive::Archive::load(&bytes)
            .map_err(|e| format!("Failed to parse archive: {e}"))?;

        let world_meta: Vec<WorldConstitutionMeta> =
            match std::fs::read("crates/constitution-app/assets/world_meta.json") {
                Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
                Err(_) => Vec::new(),
            };

        Ok((archive, world_meta))
    }
}

// build.rs generates this module
include!(concat!(env!("OUT_DIR"), "/blog_manifest.rs"));

fn built_in_posts() -> Vec<BlogPost> {
    let mut posts: Vec<BlogPost> = blog_post_sources()
        .iter()
        .filter_map(|(_slug, md)| compile_post(md))
        .collect();

    // Newest first
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    posts
}

fn compile_post(md: &str) -> Option<BlogPost> {
    let (frontmatter, body) = parse_frontmatter(md);

    let parser = pulldown_cmark::Parser::new(body);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);

    let excerpt: String = body
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#') && !l.starts_with("{{"))
        .next()
        .map(|l| {
            let s: String = l.chars().take(220).collect();
            s
        })
        .unwrap_or_default();

    Some(BlogPost {
        slug: frontmatter.get("slug")?.clone(),
        title: frontmatter.get("title")?.clone(),
        date: frontmatter.get("date").cloned().unwrap_or_default(),
        tags: frontmatter
            .get("tags")
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
        excerpt,
        html,
        markdown: body.to_string(),
        user_created: false,
    })
}

fn parse_frontmatter(md: &str) -> (std::collections::HashMap<String, String>, &str) {
    let mut map = std::collections::HashMap::new();
    if !md.starts_with("---") {
        return (map, md);
    }
    let rest = &md[3..];
    let Some(end) = rest.find("---") else {
        return (map, md);
    };
    let front = &rest[..end];
    let body = &rest[end + 3..].trim_start();
    for line in front.lines() {
        if let Some((key, val)) = line.split_once(':') {
            map.insert(key.trim().to_string(), val.trim().to_string());
        }
    }
    (map, body)
}
