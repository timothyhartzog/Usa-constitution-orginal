#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/assets/main.css") }
        main {
            class: "app-shell",
            style: "
                min-height: 100vh;
                display: grid;
                grid-template-columns: minmax(240px, 320px) minmax(0, 1fr);
                color: #17211c;
                background: #f7f5ef;
            ",
            aside {
                style: "
                    border-right: 1px solid #d8d2c3;
                    background: #fffcf4;
                    padding: 18px;
                ",
                h1 {
                    style: "font-size: 20px; line-height: 1.2; margin: 0 0 18px;",
                    "Constitution Archive"
                }
                nav {
                    style: "display: grid; gap: 8px;",
                    NavItem { label: "Search", active: true }
                    NavItem { label: "Documents", active: false }
                    NavItem { label: "Graph", active: false }
                    NavItem { label: "Vectors", active: false }
                    NavItem { label: "Pipeline", active: false }
                }
            }
            section {
                style: "padding: 20px; min-width: 0;",
                header {
                    style: "
                        display: flex;
                        align-items: center;
                        justify-content: space-between;
                        gap: 16px;
                        margin-bottom: 16px;
                    ",
                    div {
                        h2 {
                            style: "font-size: 18px; margin: 0 0 4px;",
                            "Rust Research Workbench"
                        }
                        p {
                            style: "margin: 0; color: #5f665f;",
                            "Desktop and WASM shell for search, graph exploration, vectors, and pipeline verification."
                        }
                    }
                    button {
                        style: "
                            border: 1px solid #28483a;
                            background: #28483a;
                            color: #fff;
                            border-radius: 6px;
                            padding: 9px 12px;
                        ",
                        "Verify Data"
                    }
                }
                Dashboard {}
            }
        }
    }
}

#[component]
fn NavItem(label: &'static str, active: bool) -> Element {
    let (background, color, border) = if active {
        ("#e3eee5", "#123326", "#9fbeaa")
    } else {
        ("transparent", "#4e5a52", "transparent")
    };
    rsx! {
        button {
            style: "
                width: 100%;
                text-align: left;
                border: 1px solid {border};
                background: {background};
                color: {color};
                border-radius: 6px;
                padding: 9px 10px;
            ",
            "{label}"
        }
    }
}

#[component]
fn Dashboard() -> Element {
    rsx! {
        div {
            style: "
                display: grid;
                grid-template-columns: repeat(4, minmax(160px, 1fr));
                gap: 12px;
                margin-bottom: 16px;
            ",
            StatTile { label: "Canonical Store", value: "JSON + docs" }
            StatTile { label: "Runtime", value: "Rust" }
            StatTile { label: "Targets", value: "Desktop / WASM" }
            StatTile { label: "Database Role", value: "Derived index" }
        }
        div {
            style: "
                display: grid;
                grid-template-columns: minmax(0, 1.5fr) minmax(280px, 0.8fr);
                gap: 14px;
            ",
            section {
                style: "border: 1px solid #d8d2c3; border-radius: 8px; background: #fffdf8; padding: 14px;",
                h3 { style: "margin: 0 0 10px; font-size: 16px;", "Search Surface" }
                input {
                    placeholder: "Search constitutional debates, papers, letters, clauses...",
                    style: "
                        width: 100%;
                        border: 1px solid #c8c0ad;
                        border-radius: 6px;
                        padding: 10px 12px;
                        margin-bottom: 12px;
                    ",
                }
                div {
                    style: "display: grid; gap: 8px;",
                    ResultStub { title: "BM25 + vector retrieval" }
                    ResultStub { title: "Clause and issue filters" }
                    ResultStub { title: "Citation graph expansion" }
                }
            }
            section {
                style: "border: 1px solid #d8d2c3; border-radius: 8px; background: #fffdf8; padding: 14px;",
                h3 { style: "margin: 0 0 10px; font-size: 16px;", "Pipeline Guardrails" }
                ul {
                    style: "margin: 0; padding-left: 18px; color: #47524b; line-height: 1.7;",
                    li { "Raw downloads stay on disk." }
                    li { "SurrealDB remains rebuildable." }
                    li { "Snapshots detect accidental data drift." }
                    li { "Rust stages prove parity before cutover." }
                }
            }
        }
    }
}

#[component]
fn StatTile(label: &'static str, value: &'static str) -> Element {
    rsx! {
        div {
            style: "border: 1px solid #d8d2c3; border-radius: 8px; background: #fffdf8; padding: 12px;",
            div { style: "font-size: 12px; color: #667066; margin-bottom: 6px;", "{label}" }
            div { style: "font-size: 18px; font-weight: 650;", "{value}" }
        }
    }
}

#[component]
fn ResultStub(title: &'static str) -> Element {
    rsx! {
        div {
            style: "border: 1px solid #e3ddd0; border-radius: 6px; padding: 10px; background: #fbfaf6;",
            strong { style: "display: block; font-size: 14px; margin-bottom: 4px;", "{title}" }
            span { style: "color: #667066; font-size: 13px;", "Ready for archive, vector, and graph-backed data adapters." }
        }
    }
}
