//! Citation modal: shows BibTeX / RIS / plain-text formats for a chunk,
//! lets the user copy any format or download as a file.

use constitution_archive::Chunk;
use dioxus::prelude::*;

use crate::components::url_sync::copy_to_clipboard;
use crate::export;

#[derive(Copy, Clone, PartialEq, Eq)]
enum Format {
    BibTeX,
    Ris,
    Plain,
}

impl Format {
    fn label(self) -> &'static str {
        match self {
            Self::BibTeX => "BibTeX",
            Self::Ris => "RIS",
            Self::Plain => "Plain text",
        }
    }

    fn mime(self) -> &'static str {
        match self {
            Self::BibTeX => "application/x-bibtex",
            Self::Ris => "application/x-research-info-systems",
            Self::Plain => "text/plain",
        }
    }

    fn ext(self) -> &'static str {
        match self {
            Self::BibTeX => "bib",
            Self::Ris => "ris",
            Self::Plain => "txt",
        }
    }
}

#[component]
pub fn CiteModal(chunk: Chunk, on_close: EventHandler<()>) -> Element {
    let mut format = use_signal(|| Format::BibTeX);
    let mut copied = use_signal(|| false);

    let body = match *format.read() {
        Format::BibTeX => export::chunk_bibtex(&chunk),
        Format::Ris => export::chunk_ris(&chunk),
        Format::Plain => export::chunk_citation_plain(&chunk),
    };

    let body_for_copy = body.clone();
    let chunk_for_download = chunk.clone();
    let body_for_download = body.clone();

    rsx! {
        div {
            class: "modal-overlay",
            role: "dialog",
            aria_modal: "true",
            aria_label: "Cite this passage",
            onclick: move |_| on_close.call(()),
            div { class: "modal-content cite-modal",
                onclick: move |e| e.stop_propagation(),
                div { class: "modal-header",
                    h3 { "Cite this passage" }
                    button {
                        class: "modal-close",
                        aria_label: "Close",
                        onclick: move |_| on_close.call(()),
                        "x"
                    }
                }
                div { class: "modal-body",
                    div { class: "cite-format-tabs", role: "tablist",
                        for f in [Format::BibTeX, Format::Ris, Format::Plain] {
                            {
                                let is_active = *format.read() == f;
                                rsx! {
                                    button {
                                        role: "tab",
                                        aria_selected: if is_active { "true" } else { "false" },
                                        class: if is_active { "cite-tab cite-tab-active" } else { "cite-tab" },
                                        onclick: move |_| {
                                            format.set(f);
                                            copied.set(false);
                                        },
                                        "{f.label()}"
                                    }
                                }
                            }
                        }
                    }
                    pre { class: "cite-body", "{body}" }
                    div { class: "cite-actions",
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| {
                                let payload = body_for_copy.clone();
                                copied.set(false);
                                spawn(async move {
                                    let _ = copy_to_clipboard(&payload).await;
                                });
                                copied.set(true);
                            },
                            if *copied.read() { "Copied ✓" } else { "Copy to clipboard" }
                        }
                        button {
                            class: "btn btn-secondary",
                            onclick: move |_| {
                                let f = *format.read();
                                let filename = format!(
                                    "{}.{}",
                                    crate::export::citation_key(&chunk_for_download),
                                    f.ext(),
                                );
                                let _ = crate::export::download(&filename, f.mime(), &body_for_download);
                            },
                            "Download"
                        }
                    }
                }
            }
        }
    }
}
