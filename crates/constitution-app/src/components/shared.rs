use dioxus::prelude::*;

#[component]
pub fn StatTile(label: String, value: String) -> Element {
    rsx! {
        div { class: "stat-tile",
            div { class: "stat-label", "{label}" }
            div { class: "stat-value", "{value}" }
        }
    }
}

#[component]
pub fn Tag(label: String, color: Option<String>) -> Element {
    let bg = color.unwrap_or_else(|| "#e3eee5".to_string());
    rsx! {
        span {
            class: "tag",
            style: "background: {bg};",
            "{label}"
        }
    }
}

#[component]
pub fn LoadingSpinner(message: Option<String>) -> Element {
    let archive_state = crate::state::use_archive();
    let st = archive_state.read();
    let msg = message.unwrap_or_else(|| "Loading...".to_string());
    let percent = st.progress_percent;
    let fetched_mb = st.bytes_fetched as f64 / (1024.0 * 1024.0);
    let total_mb = st.bytes_total as f64 / (1024.0 * 1024.0);
    let show_progress = st.bytes_total > 0 || st.bytes_fetched > 0;
    rsx! {
        div { class: "loading-spinner",
            div { class: "spinner-ring" }
            p { class: "spinner-message", "{msg}" }
            if show_progress {
                div { class: "progress-track",
                    div {
                        class: "progress-fill",
                        style: "width: {percent}%;",
                    }
                }
                p { class: "progress-detail",
                    if total_mb > 0.0 {
                        "{fetched_mb:.1} / {total_mb:.1} MB ({percent}%)"
                    } else {
                        "{fetched_mb:.1} MB"
                    }
                }
            }
        }
    }
}

#[component]
pub fn Modal(title: String, on_close: EventHandler<()>, children: Element) -> Element {
    rsx! {
        div { class: "modal-overlay",
            onclick: move |_| on_close.call(()),
            div { class: "modal-content",
                onclick: move |e| e.stop_propagation(),
                div { class: "modal-header",
                    h3 { "{title}" }
                    button {
                        class: "modal-close",
                        onclick: move |_| on_close.call(()),
                        "x"
                    }
                }
                div { class: "modal-body",
                    {children}
                }
            }
        }
    }
}

#[component]
pub fn EmptyState(icon: Option<String>, title: String, description: String) -> Element {
    let icon_char = icon.unwrap_or_else(|| "?".to_string());
    rsx! {
        div { class: "empty-state",
            div { class: "empty-icon", "{icon_char}" }
            h3 { "{title}" }
            p { "{description}" }
        }
    }
}
