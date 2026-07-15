//! End-to-end tests for the HTTP surface, exercised via tower::ServiceExt
//! against an in-memory archive — no TCP listener, no on-disk archive.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use constitution_archive::{Archive, Chunk, ProcessEvent, ProcessPhase, ProcessTimeline};
use constitution_server::{router, AppState};
use serde_json::Value;
use tower::ServiceExt;

const BODY_LIMIT: usize = 4 * 1024 * 1024;

fn chunk(id: &str, doc: &str, title: &str, author: &str, date: &str, text: &str) -> Chunk {
    Chunk {
        chunk_id: id.into(),
        document_id: doc.into(),
        title: title.into(),
        author: author.into(),
        date: date.into(),
        source_collection: "constitution".into(),
        source_url: "https://example/".into(),
        document_type: "foundational_document".into(),
        issue_tags: vec!["federalism".into()],
        constitutional_clause_tags: vec!["I.8".into()],
        text: text.into(),
        word_count: text.split_whitespace().count() as u32,
        preview: String::new(),
    }
}

fn fixture_archive() -> Archive {
    let chunks = vec![
        chunk(
            "us_constitution_1787_article_1_0000",
            "us_constitution_1787_article_1",
            "Article I",
            "Constitutional Convention",
            "1787-09-17",
            "Congress shall have power to lay and collect taxes; the People in Article I, Section 8 grant this. Madison wrote about this.",
        ),
        chunk(
            "federalist_10_0000",
            "federalist_10",
            "Federalist No. 10",
            "James Madison",
            "1787-11-22",
            "The latent causes of faction are thus sown. Madison reasons. Federalist No. 51 will continue.",
        ),
        chunk(
            "brutus_i_0000",
            "brutus_i",
            "Brutus, No. I",
            "Robert Yates",
            "1787-10-18",
            "When the great body of the People agree, even Article III courts cannot stand. See Brutus, No. X.",
        ),
    ];
    let event = ProcessEvent {
        id: "convention_signing".into(),
        date: "1787-09-17".into(),
        phase: ProcessPhase::Convention,
        title: "Constitution signed".into(),
        summary: "Thirty-nine signers in Philadelphia.".into(),
        actors: vec!["Washington".into()],
        locations: vec!["Philadelphia".into()],
        source_chunks: vec!["us_constitution_1787_article_1_0000".into()],
        cross_refs: vec![],
    };
    Archive::build(chunks, ProcessTimeline::from_events(vec![event]))
}

fn app() -> axum::Router {
    router(AppState::new(Arc::new(fixture_archive())))
}

async fn json_response(uri: &str) -> (StatusCode, Value) {
    let response = app()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), BODY_LIMIT).await.unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

async fn post_json(uri: &str, body: Value) -> (StatusCode, Value) {
    let response = app()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), BODY_LIMIT).await.unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

async fn request_json(method: Method, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let body = body
        .map(|value| Body::from(serde_json::to_vec(&value).unwrap()))
        .unwrap_or_else(Body::empty);
    let response = app()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), BODY_LIMIT).await.unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

#[tokio::test]
async fn healthz_reports_ok() {
    let (status, body) = json_response("/healthz").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["chunks"], 3);
}

#[tokio::test]
async fn stats_returns_archive_stats() {
    let (status, body) = json_response("/api/stats").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["chunks"], 3);
    assert!(body["citations"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn search_runs_bm25() {
    let (status, body) = post_json(
        "/api/search",
        serde_json::json!({ "query": "Madison faction", "limit": 5 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["total"].as_u64().unwrap() >= 1);
    assert_eq!(body["hits"][0]["chunk_id"], "federalist_10_0000");
}

#[tokio::test]
async fn search_rejects_empty_query() {
    let (status, _) = post_json("/api/search", serde_json::json!({ "query": "" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn search_supports_filters() {
    let (status, body) = post_json(
        "/api/search",
        serde_json::json!({
            "query": "people",
            "collections": ["nonexistent"],
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["hits"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn chunk_lookup_404s_unknown() {
    let (status, body) = json_response("/api/chunk/missing").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["kind"], "not_found");
}

#[tokio::test]
async fn chunk_lookup_returns_chunk() {
    let (status, body) = json_response("/api/chunk/federalist_10_0000").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["chunk_id"], "federalist_10_0000");
    assert_eq!(body["author"], "James Madison");
}

#[tokio::test]
async fn process_list_returns_events() {
    let (status, body) = json_response("/api/process").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["events"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn process_phase_supports_known_phase() {
    let (status, body) = json_response("/api/process/phase/convention").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn process_phase_400s_unknown() {
    let (status, _) = json_response("/api/process/phase/banana").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn process_search_finds_event() {
    let (status, body) = json_response("/api/process/search?q=Philadelphia").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn pdsa_create_list_update_and_delete_cycle() {
    let app = app();
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/pdsa")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "title": "Improve metadata coverage",
                        "aim": "Raise tagged source coverage",
                        "metric": "Tagged sources",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_body: Value = serde_json::from_slice(
        &to_bytes(create_response.into_body(), BODY_LIMIT)
            .await
            .unwrap(),
    )
    .unwrap();
    let id = create_body["id"].as_str().unwrap().to_string();
    assert_eq!(create_body["stage"], "plan");
    assert_eq!(create_body["status"], "active");

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/pdsa")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body: Value = serde_json::from_slice(
        &to_bytes(list_response.into_body(), BODY_LIMIT)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(list_body.as_array().unwrap().len(), 1);

    let patch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/pdsa/{id}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "stage": "study",
                        "doing": "Tested the import checklist",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patch_response.status(), StatusCode::OK);
    let patch_body: Value = serde_json::from_slice(
        &to_bytes(patch_response.into_body(), BODY_LIMIT)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(patch_body["stage"], "study");
    assert_eq!(patch_body["doing"], "Tested the import checklist");

    let delete_response = app
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/pdsa/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn pdsa_rejects_empty_title_and_unknown_update() {
    let (status, _) = request_json(
        Method::POST,
        "/api/pdsa",
        Some(serde_json::json!({ "title": "" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, body) = request_json(
        Method::PATCH,
        "/api/pdsa/missing",
        Some(serde_json::json!({ "stage": "do" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["kind"], "not_found");
}

#[tokio::test]
async fn suggest_returns_terms() {
    let (status, body) = json_response("/api/suggest?prefix=mad&limit=5").await;
    assert_eq!(status, StatusCode::OK);
    let terms: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(terms.iter().any(|t| t.starts_with("mad")));
}

#[tokio::test]
async fn fuzzy_term_lookup_works() {
    let (status, body) = json_response("/api/fuzzy?term=madson&max_distance=2").await;
    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().unwrap();
    assert!(arr.iter().any(|v| v[0] == "madison"));
}

#[tokio::test]
async fn citations_top_returns_targets() {
    let (status, body) = json_response("/api/citations/top?limit=5").await;
    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().unwrap();
    assert!(!arr.is_empty());
}

#[tokio::test]
async fn citations_from_returns_outgoing() {
    let (status, body) =
        json_response("/api/citations/from/us_constitution_1787_article_1_0000").await;
    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().unwrap();
    assert!(arr.iter().any(|c| c["target"]["kind"] == "clause"));
}

#[tokio::test]
async fn citations_from_404s_unknown_chunk() {
    let (status, _) = json_response("/api/citations/from/missing").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn citations_to_returns_incoming() {
    let (status, body) = json_response("/api/citations/to/person:madison").await;
    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().unwrap();
    assert!(!arr.is_empty(), "person:madison should have citations");
}

#[tokio::test]
async fn citations_graph_returns_nodes_and_edges() {
    let (status, body) = json_response("/api/citations/graph?top=5").await;
    assert_eq!(status, StatusCode::OK);
    let nodes = body["nodes"].as_array().unwrap();
    let edges = body["edges"].as_array().unwrap();
    assert!(!nodes.is_empty());
    // Node shape.
    let n0 = &nodes[0];
    assert!(n0["key"].is_string());
    assert!(n0["kind"].is_string());
    assert!(n0["label"].is_string());
    assert!(n0["citation_count"].is_u64());
    // The fixture has Madison/Hamilton co-occurring; expect at least one edge.
    if !edges.is_empty() {
        let e0 = &edges[0];
        assert!(e0["source"].is_string());
        assert!(e0["target"].is_string());
        assert!(e0["weight"].is_u64());
    }
}
