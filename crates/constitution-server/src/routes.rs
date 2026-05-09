//! Axum route definitions for the constitution server.
//!
//! The router is constructed by [`router`]. Tests build it against an
//! in-memory archive; the binary loads the archive from disk at startup.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use constitution_archive::{
    Archive, Filter, FilterValue, ProcessPhase, SearchHit, SearchOptions,
};
use serde::{Deserialize, Serialize};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::error::ApiError;
use crate::state::AppState;

/// Build the full router for `state`.
pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/healthz", get(healthz))
        .route("/api/stats", get(stats_handler))
        .route("/api/search", post(search_handler))
        .route("/api/suggest", get(suggest_handler))
        .route("/api/fuzzy", get(fuzzy_handler))
        .route("/api/chunk/:id", get(chunk_handler))
        .route("/api/process", get(process_list_handler))
        .route("/api/process/:id", get(process_get_handler))
        .route("/api/process/phase/:name", get(process_phase_handler))
        .route("/api/process/search", get(process_search_handler))
        .route("/api/citations/top", get(citations_top_handler))
        .route("/api/citations/from/:id", get(citations_from_handler))
        .route("/api/citations/to/:key", get(citations_to_handler))
        .route("/api/citations/graph", get(citations_graph_handler))
        .with_state(state.clone());

    let mut app = api;

    if let Some(dir) = &state.static_dir {
        app = app.fallback_service(ServeDir::new(dir).append_index_html_on_directories(true));
    }

    app.layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}

// ---------------------------------------------------------------------------
// /healthz
// ---------------------------------------------------------------------------

async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "chunks": state.archive.len(),
        })),
    )
}

// ---------------------------------------------------------------------------
// /api/stats
// ---------------------------------------------------------------------------

async fn stats_handler(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.archive.stats())
}

// ---------------------------------------------------------------------------
// /api/search  (POST)
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize)]
pub struct SearchRequest {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub collections: Vec<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub document_types: Vec<String>,
    #[serde(default)]
    pub issues: Vec<String>,
    #[serde(default)]
    pub clauses: Vec<String>,
    #[serde(default)]
    pub date_prefix: Option<String>,
    #[serde(default)]
    pub fuzzy_distance: Option<u32>,
    #[serde(default)]
    pub snippet_window: Option<usize>,
}

async fn search_handler(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if req.query.trim().is_empty() {
        return Err(ApiError::BadRequest("query must be non-empty".into()));
    }
    let opts = req.to_options();
    let filter = req.to_filter();
    let raw: Vec<SearchHit> = state.archive.search(&req.query, &filter, &opts);
    let archive = Arc::clone(&state.archive);
    let hits: Vec<serde_json::Value> = raw
        .iter()
        .filter_map(|h| {
            let chunk = archive.chunk(&h.chunk_id).ok()?;
            Some(serde_json::json!({
                "chunk_id": h.chunk_id,
                "title": chunk.title,
                "author": chunk.author,
                "date": chunk.date,
                "collection": chunk.source_collection,
                "source_url": chunk.source_url,
                "score": h.score,
                "matched_terms": h.matched_terms,
                "snippet": h.snippet,
            }))
        })
        .collect();
    Ok(Json(serde_json::json!({
        "total": hits.len(),
        "hits": hits,
    })))
}

impl SearchRequest {
    fn to_filter(&self) -> Filter {
        let mut f = Filter::default();
        if !self.collections.is_empty() {
            f = f.with(FilterValue::Collection(self.collections.clone()));
        }
        if !self.authors.is_empty() {
            f = f.with(FilterValue::Author(self.authors.clone()));
        }
        if !self.document_types.is_empty() {
            f = f.with(FilterValue::DocumentType(self.document_types.clone()));
        }
        if !self.issues.is_empty() {
            f = f.with(FilterValue::IssueTag(self.issues.clone()));
        }
        if !self.clauses.is_empty() {
            f = f.with(FilterValue::ClauseTag(self.clauses.clone()));
        }
        if let Some(p) = &self.date_prefix {
            f = f.with(FilterValue::DatePrefix(p.clone()));
        }
        f
    }

    fn to_options(&self) -> SearchOptions {
        SearchOptions {
            limit: self.limit.unwrap_or(25).min(200),
            min_score: 0.0,
            fuzzy_distance: self.fuzzy_distance.unwrap_or(0).min(3),
            snippet_window: self.snippet_window.unwrap_or(240).min(1024),
        }
    }
}

// ---------------------------------------------------------------------------
// /api/suggest, /api/fuzzy
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SuggestQuery {
    pub prefix: String,
    #[serde(default)]
    pub limit: Option<usize>,
}

async fn suggest_handler(
    State(state): State<AppState>,
    Query(q): Query<SuggestQuery>,
) -> Result<Json<Vec<String>>, ApiError> {
    if q.prefix.trim().is_empty() {
        return Err(ApiError::BadRequest("prefix must be non-empty".into()));
    }
    let limit = q.limit.unwrap_or(10).min(50);
    Ok(Json(state.archive.suggest(&q.prefix, limit)))
}

#[derive(Debug, Deserialize)]
pub struct FuzzyQuery {
    pub term: String,
    #[serde(default)]
    pub max_distance: Option<u32>,
    #[serde(default)]
    pub limit: Option<usize>,
}

async fn fuzzy_handler(
    State(state): State<AppState>,
    Query(q): Query<FuzzyQuery>,
) -> Result<Json<Vec<(String, u32)>>, ApiError> {
    if q.term.trim().is_empty() {
        return Err(ApiError::BadRequest("term must be non-empty".into()));
    }
    let max_distance = q.max_distance.unwrap_or(2).min(3);
    let limit = q.limit.unwrap_or(10).min(50);
    Ok(Json(state.archive.fuzzy_terms(&q.term, max_distance, limit)))
}

// ---------------------------------------------------------------------------
// /api/chunk/:id
// ---------------------------------------------------------------------------

async fn chunk_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<constitution_archive::Chunk>, ApiError> {
    let chunk = state.archive.chunk(&id)?.clone();
    Ok(Json(chunk))
}

// ---------------------------------------------------------------------------
// /api/process
// ---------------------------------------------------------------------------

async fn process_list_handler(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "events": state.archive.timeline().events,
    }))
}

async fn process_get_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<constitution_archive::ProcessEvent>, ApiError> {
    let event = state.archive.timeline().get(&id)?.clone();
    Ok(Json(event))
}

async fn process_phase_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Vec<constitution_archive::ProcessEvent>>, ApiError> {
    let phase: ProcessPhase = serde_json::from_value(serde_json::Value::String(name.clone()))
        .map_err(|_| ApiError::BadRequest(format!("unknown phase: {name}")))?;
    let events: Vec<_> = state.archive.timeline().by_phase(phase).into_iter().cloned().collect();
    Ok(Json(events))
}

#[derive(Debug, Deserialize)]
pub struct ProcessSearchQuery {
    pub q: String,
}

async fn process_search_handler(
    State(state): State<AppState>,
    Query(qq): Query<ProcessSearchQuery>,
) -> Result<Json<Vec<constitution_archive::ProcessEvent>>, ApiError> {
    if qq.q.trim().is_empty() {
        return Err(ApiError::BadRequest("q must be non-empty".into()));
    }
    let events: Vec<_> = state.archive.timeline().search(&qq.q).into_iter().cloned().collect();
    Ok(Json(events))
}

// ---------------------------------------------------------------------------
// /api/citations/*
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TopCitationsQuery {
    #[serde(default)]
    pub limit: Option<usize>,
}

async fn citations_top_handler(
    State(state): State<AppState>,
    Query(q): Query<TopCitationsQuery>,
) -> Json<Vec<(String, usize)>> {
    let limit = q.limit.unwrap_or(25).min(500);
    Json(state.archive.top_citation_targets(limit))
}

async fn citations_from_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<constitution_archive::Citation>>, ApiError> {
    let citations = state
        .archive
        .citations_from(&id)?
        .into_iter()
        .cloned()
        .collect();
    Ok(Json(citations))
}

#[derive(Serialize)]
struct CitedByRow {
    chunk: constitution_archive::Chunk,
    citation: constitution_archive::Citation,
}

async fn citations_to_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Query(q): Query<TopCitationsQuery>,
) -> Json<Vec<CitedByRow>> {
    let limit = q.limit.unwrap_or(50).min(500);
    let rows: Vec<CitedByRow> = state
        .archive
        .cited_by(&key)
        .into_iter()
        .take(limit)
        .map(|(chunk, citation)| CitedByRow {
            chunk: chunk.clone(),
            citation: citation.clone(),
        })
        .collect();
    Json(rows)
}

#[derive(Debug, Deserialize)]
pub struct CitationGraphQuery {
    #[serde(default)]
    pub top: Option<usize>,
}

async fn citations_graph_handler(
    State(state): State<AppState>,
    Query(q): Query<CitationGraphQuery>,
) -> Json<constitution_archive::CitationGraphView> {
    let top = q.top.unwrap_or(30).clamp(2, 200);
    Json(state.archive.citation_graph_view(top))
}

/// Convenience: build a router with no static-file fallback.
pub fn router_for(archive: Arc<Archive>) -> Router {
    router(AppState::new(archive))
}
