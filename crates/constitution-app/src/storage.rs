//! Persistent storage shim. Uses `localStorage` on WASM, in-memory on native.

#[cfg(target_arch = "wasm32")]
pub fn get(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item(key).ok()?
}

#[cfg(target_arch = "wasm32")]
pub fn set(key: &str, value: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return false;
    };
    storage.set_item(key, value).is_ok()
}

#[cfg(target_arch = "wasm32")]
pub fn remove(key: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return false;
    };
    storage.remove_item(key).is_ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get(_key: &str) -> Option<String> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set(_key: &str, _value: &str) -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn remove(_key: &str) -> bool {
    false
}

pub const KEY_DRAFT: &str = "constitution-app:blog-draft";
pub const KEY_POSTS: &str = "constitution-app:blog-posts";
pub const KEY_THEME: &str = "constitution-app:theme";
pub const KEY_USER_DATA: &str = "constitution-app:user-data";
pub const KEY_PDSA: &str = "constitution-app:pdsa-cycles";
