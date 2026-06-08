#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

mod components;
mod export;
mod router;
mod state;
mod storage;
mod url_state;

use std::rc::Rc;

use dioxus::prelude::*;

use components::command_palette::CommandPalette;
use components::nav::Sidebar;
use components::shortcuts::GlobalShortcuts;
use components::url_sync::UrlSync;
use router::Route;
use state::{
    ArchiveState, BlogDraft, BlogPost, BlogState, CommandPaletteState, SearchState, SelectionState,
    ShortcutsState, Theme, UserData, UserDataPersisted, WorldConstitutionMeta,
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
    use_context_provider(|| Signal::new(load_initial_user_data()));
    use_context_provider(|| Signal::new(CommandPaletteState::default()));
    use_context_provider(|| Signal::new(ShortcutsState::default()));

    let mut archive_state = state::use_archive();

    use_future(move || async move {
        let progress_update = move |fetched: u64, total: u64| {
            let mut state = archive_state.write();
            state.bytes_fetched = fetched;
            state.bytes_total = total;
            state.progress_percent = if total > 0 {
                ((fetched.saturating_mul(100)) / total).min(100) as u8
            } else {
                0
            };
        };

        match load_archive_data(progress_update).await {
            Ok((archive, world_meta)) => {
                let mut state = archive_state.write();
                state.archive = Some(Rc::new(archive));
                state.world_meta = world_meta;
                state.loading = false;
                state.error = None;
                state.progress_percent = 100;
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
        GlobalShortcuts {}
        CommandPalette {}
        UrlSync {}
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

fn load_initial_user_data() -> UserData {
    let persisted: UserDataPersisted = storage::get(storage::KEY_USER_DATA)
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    UserData {
        history: persisted.history,
        bookmarks: persisted.bookmarks,
        recent_searches: persisted.recent_searches,
        annotations: persisted.annotations,
        next_annotation_seq: persisted.next_annotation_seq,
    }
}

/// Persist `UserData` to localStorage. Best-effort; failures are logged
/// only in debug builds and never propagated.
pub fn persist_user_data(data: &UserData) {
    let p = UserDataPersisted {
        history: data.history.clone(),
        bookmarks: data.bookmarks.clone(),
        recent_searches: data.recent_searches.clone(),
        annotations: data.annotations.clone(),
        next_annotation_seq: data.next_annotation_seq,
    };
    if let Ok(json) = serde_json::to_string(&p) {
        storage::set(storage::KEY_USER_DATA, &json);
    }
}

/// Returns today's date as ISO-8601 ("YYYY-MM-DD"). Web build reads from
/// JS Date; native build returns a build-time constant.
pub fn today_iso() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let date = js_sys::Date::new_0();
        let y = date.get_full_year() as i32;
        let m = date.get_month() as u32 + 1;
        let d = date.get_date() as u32;
        return format!("{y:04}-{m:02}-{d:02}");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "2026-05-30".to_string()
    }
}

async fn load_archive_data(
    mut on_progress: impl FnMut(u64, u64) + 'static,
) -> Result<(constitution_archive::Archive, Vec<WorldConstitutionMeta>), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let archive_bytes = fetch_with_progress(
            "assets/constitution_archive.bin",
            &mut on_progress,
        )
        .await?;

        let archive = constitution_archive::Archive::load(&archive_bytes)
            .map_err(|e| format!("Failed to parse archive: {e}"))?;

        use gloo_net::http::Request;
        let world_meta: Vec<WorldConstitutionMeta> = match Request::get("assets/world_meta.json")
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
        let _ = on_progress;
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

#[cfg(target_arch = "wasm32")]
async fn fetch_with_progress(
    url: &str,
    on_progress: &mut (impl FnMut(u64, u64) + 'static),
) -> Result<Vec<u8>, String> {
    use js_sys::Uint8Array;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};

    let window = web_sys::window().ok_or("no window")?;

    let opts = RequestInit::new();
    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to build request: {e:?}"))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {e:?}"))?;
    let response: Response = resp_value
        .dyn_into()
        .map_err(|_| "Response cast failed".to_string())?;

    if !response.ok() {
        return Err(format!("HTTP {} fetching archive", response.status()));
    }

    let total: u64 = response
        .headers()
        .get("content-length")
        .ok()
        .flatten()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let body = response.body().ok_or("response has no body")?;
    let reader_value = body.get_reader();
    let reader: web_sys::ReadableStreamDefaultReader = reader_value
        .dyn_into()
        .map_err(|_| "reader cast failed".to_string())?;

    let mut buf: Vec<u8> = if total > 0 {
        Vec::with_capacity(total as usize)
    } else {
        Vec::new()
    };

    loop {
        let chunk = JsFuture::from(reader.read())
            .await
            .map_err(|e| format!("Stream read failed: {e:?}"))?;
        let obj: js_sys::Object = chunk
            .dyn_into()
            .map_err(|_| "stream result cast failed".to_string())?;
        let done = js_sys::Reflect::get(&obj, &"done".into())
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false);
        if done {
            break;
        }
        let value = js_sys::Reflect::get(&obj, &"value".into())
            .map_err(|_| "no value in stream chunk".to_string())?;
        let arr = Uint8Array::new(&value);
        let mut piece = vec![0u8; arr.length() as usize];
        arr.copy_to(&mut piece);
        buf.extend_from_slice(&piece);
        on_progress(buf.len() as u64, total);
    }

    Ok(buf)
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
