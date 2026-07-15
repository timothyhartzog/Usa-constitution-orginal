//! Axum route definitions for the constitution server.
//!
//! The router is constructed by [`router`]. Tests build it against an
//! in-memory archive; the binary loads the archive from disk at startup.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use constitution_archive::{Archive, Filter, FilterValue, ProcessPhase, SearchHit, SearchOptions};
use serde::{Deserialize, Serialize};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::error::ApiError;
use crate::state::AppState;

/// Build the full router for `state`.
pub fn router(state: AppState) -> Router {
    let mut api = Router::new()
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
        .route(
            "/api/pdsa",
            get(pdsa_list_handler).post(pdsa_create_handler),
        )
        .route(
            "/api/pdsa/:id",
            get(pdsa_get_handler)
                .patch(pdsa_update_handler)
                .delete(pdsa_delete_handler),
        )
        .route("/api/citations/top", get(citations_top_handler))
        .route("/api/citations/from/:id", get(citations_from_handler))
        .route("/api/citations/to/:key", get(citations_to_handler))
        .route("/api/citations/graph", get(citations_graph_handler))
        .route("/api/graph/connections/:id", get(graph_connections_handler))
        .route("/api/graph/ego/:id", get(graph_ego_handler))
        .route("/api/graph/path", get(graph_path_handler));

    #[cfg(feature = "ml")]
    {
        api = api.route("/api/rag", get(rag_handler));
    }

    let mut app = api.with_state(state.clone());

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
    Ok(Json(state.archive.fuzzy_terms(
        &q.term,
        max_distance,
        limit,
    )))
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
    let events: Vec<_> = state
        .archive
        .timeline()
        .by_phase(phase)
        .into_iter()
        .cloned()
        .collect();
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
    let events: Vec<_> = state
        .archive
        .timeline()
        .search(&qq.q)
        .into_iter()
        .cloned()
        .collect();
    Ok(Json(events))
}

// ---------------------------------------------------------------------------
// /api/pdsa
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PdsaStage {
    #[default]
    Plan,
    Do,
    Study,
    Act,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PdsaStatus {
    #[default]
    Active,
    Complete,
    Archived,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdsaCycle {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub aim: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub metric: String,
    #[serde(default)]
    pub baseline: String,
    #[serde(default)]
    pub target: String,
    #[serde(default)]
    pub plan: String,
    #[serde(default)]
    pub prediction: String,
    #[serde(default)]
    pub doing: String,
    #[serde(default)]
    pub study: String,
    #[serde(default)]
    pub act: String,
    #[serde(default)]
    pub stage: PdsaStage,
    #[serde(default)]
    pub status: PdsaStatus,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct PdsaCreateRequest {
    pub title: String,
    #[serde(default)]
    pub aim: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub metric: String,
    #[serde(default)]
    pub baseline: String,
    #[serde(default)]
    pub target: String,
    #[serde(default)]
    pub plan: String,
    #[serde(default)]
    pub prediction: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct PdsaUpdateRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub aim: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub metric: Option<String>,
    #[serde(default)]
    pub baseline: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub plan: Option<String>,
    #[serde(default)]
    pub prediction: Option<String>,
    #[serde(default)]
    pub doing: Option<String>,
    #[serde(default)]
    pub study: Option<String>,
    #[serde(default)]
    pub act: Option<String>,
    #[serde(default)]
    pub stage: Option<PdsaStage>,
    #[serde(default)]
    pub status: Option<PdsaStatus>,
}

async fn pdsa_list_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<PdsaCycle>>, ApiError> {
    let cycles = state
        .pdsa_cycles
        .read()
        .map_err(|_| ApiError::Internal("PDSA store is unavailable".into()))?
        .clone();
    Ok(Json(cycles))
}

async fn pdsa_create_handler(
    State(state): State<AppState>,
    Json(req): Json<PdsaCreateRequest>,
) -> Result<(StatusCode, Json<PdsaCycle>), ApiError> {
    let title = req.title.trim().to_string();
    if title.is_empty() {
        return Err(ApiError::BadRequest("title must be non-empty".into()));
    }

    let timestamp = unix_timestamp_string();
    let mut cycles = state
        .pdsa_cycles
        .write()
        .map_err(|_| ApiError::Internal("PDSA store is unavailable".into()))?;
    let id = unique_pdsa_id(&title, cycles.len(), &timestamp);
    let cycle = PdsaCycle {
        id,
        title,
        aim: req.aim,
        owner: req.owner,
        metric: req.metric,
        baseline: req.baseline,
        target: req.target,
        plan: req.plan,
        prediction: req.prediction,
        doing: String::new(),
        study: String::new(),
        act: String::new(),
        stage: PdsaStage::Plan,
        status: PdsaStatus::Active,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    };
    cycles.insert(0, cycle.clone());
    Ok((StatusCode::CREATED, Json(cycle)))
}

async fn pdsa_get_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PdsaCycle>, ApiError> {
    let cycles = state
        .pdsa_cycles
        .read()
        .map_err(|_| ApiError::Internal("PDSA store is unavailable".into()))?;
    let cycle = cycles
        .iter()
        .find(|cycle| cycle.id == id)
        .cloned()
        .ok_or_else(|| ApiError::NotFound(format!("PDSA cycle {id}")))?;
    Ok(Json(cycle))
}

async fn pdsa_update_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<PdsaUpdateRequest>,
) -> Result<Json<PdsaCycle>, ApiError> {
    let mut cycles = state
        .pdsa_cycles
        .write()
        .map_err(|_| ApiError::Internal("PDSA store is unavailable".into()))?;
    let cycle = cycles
        .iter_mut()
        .find(|cycle| cycle.id == id)
        .ok_or_else(|| ApiError::NotFound(format!("PDSA cycle {id}")))?;

    if let Some(title) = req.title {
        let trimmed = title.trim().to_string();
        if trimmed.is_empty() {
            return Err(ApiError::BadRequest("title must be non-empty".into()));
        }
        cycle.title = trimmed;
    }
    if let Some(value) = req.aim {
        cycle.aim = value;
    }
    if let Some(value) = req.owner {
        cycle.owner = value;
    }
    if let Some(value) = req.metric {
        cycle.metric = value;
    }
    if let Some(value) = req.baseline {
        cycle.baseline = value;
    }
    if let Some(value) = req.target {
        cycle.target = value;
    }
    if let Some(value) = req.plan {
        cycle.plan = value;
    }
    if let Some(value) = req.prediction {
        cycle.prediction = value;
    }
    if let Some(value) = req.doing {
        cycle.doing = value;
    }
    if let Some(value) = req.study {
        cycle.study = value;
    }
    if let Some(value) = req.act {
        cycle.act = value;
    }
    if let Some(value) = req.stage {
        cycle.stage = value;
    }
    if let Some(value) = req.status {
        cycle.status = value;
    }
    cycle.updated_at = unix_timestamp_string();

    Ok(Json(cycle.clone()))
}

async fn pdsa_delete_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let mut cycles = state
        .pdsa_cycles
        .write()
        .map_err(|_| ApiError::Internal("PDSA store is unavailable".into()))?;
    let original_len = cycles.len();
    cycles.retain(|cycle| cycle.id != id);
    if cycles.len() == original_len {
        return Err(ApiError::NotFound(format!("PDSA cycle {id}")));
    }
    Ok(StatusCode::NO_CONTENT)
}

fn unix_timestamp_string() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs().to_string(),
        Err(_) => "0".to_string(),
    }
}

fn unique_pdsa_id(title: &str, count: usize, timestamp: &str) -> String {
    format!("pdsa-{timestamp}-{count}-{}", slugify(title))
}

fn slugify(title: &str) -> String {
    let mut out = String::new();
    let mut last_dash = true;
    for ch in title.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "cycle".to_string()
    } else {
        out
    }
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

// ---------------------------------------------------------------------------
// /api/graph/* (Knowledge Graph)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct GraphQuery {
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub depth: Option<usize>,
    #[serde(default)]
    pub max_nodes: Option<usize>,
}

async fn graph_connections_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<GraphQuery>,
) -> Result<Json<Vec<constitution_archive::graph::KgEdge>>, ApiError> {
    let kg = state
        .knowledge_graph
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Knowledge Graph not loaded".to_string()))?;
    let limit = q.limit.unwrap_or(50).min(200);
    Ok(Json(kg.connections_for(&id, limit)))
}

#[derive(Serialize)]
pub struct EgoGraphResponse {
    pub nodes: Vec<constitution_archive::graph::KgNode>,
    pub edges: Vec<constitution_archive::graph::KgEdge>,
}

async fn graph_ego_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<GraphQuery>,
) -> Result<Json<EgoGraphResponse>, ApiError> {
    let kg = state
        .knowledge_graph
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Knowledge Graph not loaded".to_string()))?;
    let depth = q.depth.unwrap_or(2).min(4);
    let max_nodes = q.max_nodes.unwrap_or(100).min(500);

    let (nodes, edges) = kg.ego_graph(&id, depth, max_nodes);
    Ok(Json(EgoGraphResponse { nodes, edges }))
}

#[derive(Debug, Deserialize)]
pub struct PathQuery {
    pub start: String,
    pub end: String,
}

async fn graph_path_handler(
    State(state): State<AppState>,
    Query(q): Query<PathQuery>,
) -> Result<Json<Option<Vec<String>>>, ApiError> {
    let kg = state
        .knowledge_graph
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Knowledge Graph not loaded".to_string()))?;
    Ok(Json(kg.shortest_path(&q.start, &q.end)))
}

/// Convenience: build a router with no static-file fallback.
pub fn router_for(archive: Arc<Archive>) -> Router {
    router(AppState::new(archive))
}

#[cfg(feature = "ml")]
use crate::rag_types::RagQuery;
#[cfg(feature = "ml")]
use axum::response::sse::{Event, Sse};
#[cfg(feature = "ml")]
use futures_util::stream::Stream;
#[cfg(feature = "ml")]
use std::convert::Infallible;

#[cfg(feature = "ml")]
async fn rag_handler(
    State(state): State<AppState>,
    Query(req): Query<RagQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let engine = state
        .rag_engine
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("RAG engine not configured".to_string()))?;

    let mut contexts = Vec::new();
    let ids: Vec<&str> = req
        .context_ids
        .split(',')
        .filter(|s| !s.is_empty())
        .collect();
    for id in ids {
        if let Ok(chunk) = state.archive.chunk(id) {
            contexts.push((id.to_string(), chunk.text.clone()));
        }
    }

    let config = constitution_archive::ml::RagConfig::default();

    // Stub implementation: In a real system, the candle loop would run on a blocking thread
    // and yield tokens via an mpsc channel to the SSE stream.
    // Here we generate the full answer (using the stub) and stream it in chunks for demonstration.
    let full_answer = engine
        .generate_answer(&req.query, &contexts, &config)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Simulate streaming by breaking into words
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    tokio::spawn(async move {
        for word in full_answer.split_whitespace() {
            let msg = format!("{} ", word);
            if tx.send(Ok(Event::default().data(msg))).await.is_err() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        let _ = tx.send(Ok(Event::default().event("end").data(""))).await;
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(1))
            .text("keep-alive-text"),
    ))
}
