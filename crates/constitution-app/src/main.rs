#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

mod components;
mod router;
mod state;

use dioxus::prelude::*;

use components::nav::Sidebar;
use router::Route;
use state::{ArchiveState, BlogState, SearchState, SelectionState};

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
    use_context_provider(|| Signal::new(BlogState::default()));

    let mut archive_state = state::use_archive();

    use_future(move || async move {
        let mut state = archive_state.write();
        state.loading = false;
        state.error = Some("Archive loading requires WASM runtime. Run with `dx serve --features web`.".to_string());
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
