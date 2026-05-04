//! HTTP request handlers for the API

use crate::error_response::ApiError;
use crate::state::AppState;
use actix_web::{web, HttpResponse};
use constitutional_lib::types;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Search request payload
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub search_type: Option<String>,
    pub max_results: Option<usize>,
    pub filters: Option<types::SearchFilters>,
}

/// Search response
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<types::SearchResult>,
    pub count: usize,
    pub search_type: String,
}

/// Document ingestion request
#[derive(Debug, Deserialize)]
pub struct IngestRequest {
    pub title: String,
    pub author: Option<String>,
    pub date: Option<String>,
    pub source_collection: String,
    pub source_url: Option<String>,
    pub document_type: String,
    pub text: String,
}

/// Document ingestion response
#[derive(Debug, Serialize)]
pub struct IngestResponse {
    pub document_id: String,
    pub chunks: usize,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Health check endpoint
pub async fn health_handler() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Generic search endpoint (dispatches to appropriate search type)
pub async fn search_handler(
    state: web::Data<Arc<AppState>>,
    req: web::Json<SearchRequest>,
) -> Result<HttpResponse, ApiError> {
    let search_type = req.search_type.as_deref().unwrap_or("fulltext");

    let results = match search_type {
        "fulltext" => state.search_fulltext(&req.query)?,
        "fuzzy" => state.search_fuzzy(&req.query, 2)?,
        _ => return Err(ApiError::InvalidRequest(format!(
            "Unknown search type: {}",
            search_type
        ))),
    };

    let count = results.len();
    let max_results = req.max_results.unwrap_or(50).min(results.len());

    Ok(HttpResponse::Ok().json(SearchResponse {
        results: results.into_iter().take(max_results).collect(),
        count,
        search_type: search_type.to_string(),
    }))
}

/// Full-text search endpoint
pub async fn search_fulltext_handler(
    state: web::Data<Arc<AppState>>,
    req: web::Json<SearchRequest>,
) -> Result<HttpResponse, ApiError> {
    let results = state.search_fulltext(&req.query)?;
    let count = results.len();
    let max_results = req.max_results.unwrap_or(50).min(results.len());

    Ok(HttpResponse::Ok().json(SearchResponse {
        results: results.into_iter().take(max_results).collect(),
        count,
        search_type: "fulltext".to_string(),
    }))
}

/// Fuzzy search endpoint
pub async fn search_fuzzy_handler(
    state: web::Data<Arc<AppState>>,
    req: web::Json<SearchRequest>,
) -> Result<HttpResponse, ApiError> {
    let results = state.search_fuzzy(&req.query, 2)?;
    let count = results.len();
    let max_results = req.max_results.unwrap_or(50).min(results.len());

    Ok(HttpResponse::Ok().json(SearchResponse {
        results: results.into_iter().take(max_results).collect(),
        count,
        search_type: "fuzzy".to_string(),
    }))
}

/// Semantic search endpoint
pub async fn search_semantic_handler() -> HttpResponse {
    HttpResponse::NotImplemented().json(
        serde_json::json!({"error": "Semantic search requires embeddings (Phase 2+)"})
    )
}

/// Document ingestion endpoint
pub async fn ingest_document_handler(
    state: web::Data<Arc<AppState>>,
    req: web::Json<IngestRequest>,
) -> Result<HttpResponse, ApiError> {
    let doc = types::Document {
        id: types::DocumentId::new(uuid::Uuid::new_v4().to_string()),
        title: req.title.clone(),
        author: req.author.clone(),
        date: req.date.clone(),
        source_collection: req.source_collection.clone(),
        source_url: req.source_url.clone(),
        document_type: req.document_type.clone(),
        text: req.text.clone(),
        metadata: std::collections::HashMap::new(),
    };

    let doc_id = doc.id.to_string();
    state.ingest_document(doc)?;

    Ok(HttpResponse::Created().json(IngestResponse {
        document_id: doc_id,
        chunks: 0, // TODO: Return actual chunk count
    }))
}

/// Get document endpoint
pub async fn get_document_handler(
    state: web::Data<Arc<AppState>>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let doc_id = path.into_inner();
    state
        .get_document(&doc_id)
        .map(|doc| HttpResponse::Ok().json(doc))
        .ok_or_else(|| ApiError::NotFound(format!("Document not found: {}", doc_id)))
}

/// Delete document endpoint
pub async fn delete_document_handler(path: web::Path<String>) -> HttpResponse {
    let _doc_id = path.into_inner();
    // TODO: Implement document deletion
    HttpResponse::NoContent().finish()
}

/// Get index statistics endpoint
pub async fn get_index_stats_handler(
    state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    let stats = state.get_stats();
    HttpResponse::Ok().json(stats)
}

/// Export index endpoint
pub async fn export_index_handler(
    state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    let stats = state.get_stats();
    HttpResponse::Ok().json(serde_json::json!({
        "status": "not_implemented",
        "message": "Index export requires serialization support (Phase 2+)",
        "stats": stats
    }))
}
