//! Server-shared state: an `Arc<Archive>` plus a path for static-file serving.

use std::path::PathBuf;
use std::sync::Arc;

use constitution_archive::Archive;

/// Cheaply-cloneable handle to the loaded archive plus configuration.
#[derive(Clone)]
pub struct AppState {
    /// Loaded, queryable archive shared by every handler.
    pub archive: Arc<Archive>,
    /// Optional directory to expose at `/`. Typically the repository's
    /// `frontend/` directory so the WASM page can be served by the same
    /// process as the API.
    pub static_dir: Option<PathBuf>,
}

impl AppState {
    /// Construct a new state from an arc'd archive.
    pub fn new(archive: Arc<Archive>) -> Self {
        Self {
            archive,
            static_dir: None,
        }
    }

    /// Builder: enable static-file serving from `dir`.
    pub fn with_static_dir(mut self, dir: PathBuf) -> Self {
        self.static_dir = Some(dir);
        self
    }
}
