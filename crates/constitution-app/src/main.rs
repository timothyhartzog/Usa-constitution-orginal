#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

mod components;
mod router;
mod state;

use std::rc::Rc;

use dioxus::prelude::*;

use components::nav::Sidebar;
use router::Route;
use state::{ArchiveState, BlogState, BlogPost, SearchState, SelectionState, WorldConstitutionMeta};

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
    use_context_provider(|| Signal::new(BlogState {
        posts: built_in_posts(),
        ..Default::default()
    }));

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

    rsx! {
        document::Stylesheet { href: asset!("/assets/main.css") }
        main { class: "app-shell",
            Sidebar {}
            section { class: "app-content",
                Router::<Route> {}
            }
        }
    }
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

fn built_in_posts() -> Vec<BlogPost> {
    let md = include_str!("../../../content/blog/welcome.md");
    let (frontmatter, body) = parse_frontmatter(md);

    let parser = pulldown_cmark::Parser::new(body);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);

    let excerpt: String = body.chars().take(200).collect();

    vec![BlogPost {
        slug: frontmatter.get("slug").cloned().unwrap_or_else(|| "welcome".into()),
        title: frontmatter.get("title").cloned().unwrap_or_else(|| "Welcome".into()),
        date: frontmatter.get("date").cloned().unwrap_or_default(),
        tags: frontmatter
            .get("tags")
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
        excerpt,
        html,
    }]
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
