use crate::state::use_archive;
use dioxus::prelude::*;

#[component]
pub fn ChatPage() -> Element {
    let mut query = use_signal(|| String::new());
    let mut messages = use_signal(|| Vec::<(String, String)>::new());
    let mut is_loading = use_signal(|| false);
    let archive_state = use_archive();

    let submit_query = move |_| {
        let q = query.read().clone();
        if q.is_empty() {
            return;
        }

        messages.write().push(("User".to_string(), q.clone()));
        query.set(String::new());
        is_loading.set(true);

        // Perform BM25 search to get contexts
        let mut context_ids_str = String::new();
        {
            let state = archive_state.read();
            let hits = state.search(&q, &Default::default(), &Default::default());
            let top_ids: Vec<String> = hits.into_iter().take(5).map(|h| h.chunk_id).collect();
            context_ids_str = top_ids.join(",");
        }

        spawn(async move {
            let _url = format!(
                "/api/rag?query={}&context_ids={}",
                urlencoding::encode(&q),
                urlencoding::encode(&context_ids_str)
            );

            #[cfg(target_arch = "wasm32")]
            {
                use futures_util::StreamExt;
                use gloo_net::eventsource::futures::EventSource;

                let mut es = match EventSource::new(&_url) {
                    Ok(es) => es,
                    Err(_) => {
                        messages.write().push((
                            "System".to_string(),
                            "Failed to connect to RAG server.".to_string(),
                        ));
                        is_loading.set(false);
                        return;
                    }
                };

                messages
                    .write()
                    .push(("Archive".to_string(), String::new()));

                if let Ok(mut stream) = es.subscribe("message") {
                    while let Some(Ok((_, event))) = stream.next().await {
                        if let Some(data) = event.data().as_string() {
                            let mut msgs = messages.write();
                            let last_idx = msgs.len() - 1;
                            msgs[last_idx].1.push_str(&data);
                        }
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                messages.write().push((
                    "Archive".to_string(),
                    "[Native Client] Semantic search and SSE streaming are routed via the Web API."
                        .to_string(),
                ));
            }

            is_loading.set(false);
        });
    };

    rsx! {
        div { class: "page chat-page",
            header { class: "page-header",
                h2 { "Chat with the Archive" }
                p { class: "page-subtitle", "Ask a question and the AI will answer using only historical sources." }
            }
            div { class: "chat-container", style: "display: flex; flex-direction: column; height: 60vh; border: 1px solid #ccc; padding: 1rem;",
                div { class: "chat-history", style: "flex-grow: 1; overflow-y: auto; margin-bottom: 1rem;",
                    for (role, msg) in messages.read().iter() {
                        div { class: "chat-message {role.to_lowercase()}", style: "margin-bottom: 0.5rem;",
                            strong { "{role}: " }
                            span { "{msg}" }
                        }
                    }
                    if *is_loading.read() {
                        div { class: "chat-message system",
                            em { "Analyzing sources and typing..." }
                        }
                    }
                }
                div { class: "chat-input-area", style: "display: flex; gap: 0.5rem;",
                    input {
                        r#type: "text",
                        placeholder: "e.g., How did the Founders view executive war powers?",
                        value: "{query}",
                        oninput: move |e| query.set(e.value().clone()),
                        onkeydown: move |e| {
                            if e.key() == dioxus::prelude::Key::Enter {
                                // Can't easily call submit_query here due to move semantics, relying on button
                            }
                        },
                        style: "flex-grow: 1; padding: 0.5rem;"
                    }
                    button {
                        onclick: submit_query,
                        disabled: *is_loading.read(),
                        style: "padding: 0.5rem 1rem;",
                        "Ask"
                    }
                }
            }
        }
    }
}
