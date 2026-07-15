//! HTTP request handlers for the API

use crate::error_response::ApiError;
use crate::state::AppState;
use actix_web::{web, HttpResponse};
use constitutional_lib::types;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        health_handler,
        search_handler,
        search_fulltext_handler,
        search_fuzzy_handler,
        search_semantic_handler,
        ingest_document_handler,
        bulk_ingest_documents_handler,
        get_document_handler,
        delete_document_handler,
        get_index_stats_handler,
        export_index_handler
    ),
    components(
        schemas(SearchRequest, SearchResponse, IngestRequest, IngestResponse, BulkIngestRequest, BulkIngestResponse, HealthResponse)
    ),
    tags(
        (name = "search", description = "Search endpoints"),
        (name = "documents", description = "Document management"),
        (name = "system", description = "System operations")
    )
)]
pub struct ApiDoc;

/// Search request payload
#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchRequest {
    pub query: String,
    pub search_type: Option<String>,
    pub max_results: Option<usize>,
    #[schema(value_type = Option<Object>)]
    pub filters: Option<types::SearchFilters>,
}

/// Search response
#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResponse {
    #[schema(value_type = Vec<Object>)]
    pub results: Vec<types::SearchResult>,
    pub count: usize,
    pub search_type: String,
}

/// Document ingestion request
#[derive(Debug, Deserialize, ToSchema)]
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
#[derive(Debug, Serialize, ToSchema)]
pub struct IngestResponse {
    pub document_id: String,
    pub chunks: usize,
}

/// Bulk ingestion request
#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkIngestRequest {
    pub documents: Vec<IngestRequest>,
}

/// Bulk ingestion response
#[derive(Debug, Serialize, ToSchema)]
pub struct BulkIngestResponse {
    pub document_ids: Vec<String>,
    pub total_chunks: usize,
}

/// Health check response
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "system",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health_handler() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Generic search endpoint (dispatches to appropriate search type)
#[utoipa::path(
    post,
    path = "/api/search",
    tag = "search",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search results", body = SearchResponse),
        (status = 400, description = "Invalid request")
    )
)]
pub async fn search_handler(
    state: web::Data<Arc<AppState>>,
    req: web::Json<SearchRequest>,
) -> Result<HttpResponse, ApiError> {
    let search_type = req.search_type.as_deref().unwrap_or("fulltext");

    let results = match search_type {
        "fulltext" => state.search_fulltext(&req.query)?,
        "fuzzy" => state.search_fuzzy(&req.query, 2)?,
        "semantic" => state.search_semantic(&req.query)?,
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
#[utoipa::path(
    post,
    path = "/api/search/fulltext",
    tag = "search",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search results", body = SearchResponse)
    )
)]
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
#[utoipa::path(
    post,
    path = "/api/search/fuzzy",
    tag = "search",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search results", body = SearchResponse)
    )
)]
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
#[utoipa::path(
    post,
    path = "/api/search/semantic",
    tag = "search",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search results", body = SearchResponse)
    )
)]
pub async fn search_semantic_handler(
    state: web::Data<Arc<AppState>>,
    req: web::Json<SearchRequest>,
) -> Result<HttpResponse, ApiError> {
    let results = state.search_semantic(&req.query)?;
    let count = results.len();
    let max_results = req.max_results.unwrap_or(50).min(results.len());

    Ok(HttpResponse::Ok().json(SearchResponse {
        results: results.into_iter().take(max_results).collect(),
        count,
        search_type: "semantic".to_string(),
    }))
}

/// Document ingestion endpoint
#[utoipa::path(
    post,
    path = "/api/documents",
    tag = "documents",
    request_body = IngestRequest,
    responses(
        (status = 201, description = "Document ingested", body = IngestResponse)
    )
)]
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
    let chunks_added = state.ingest_document(doc)?;

    Ok(HttpResponse::Created().json(IngestResponse {
        document_id: doc_id,
        chunks: chunks_added,
    }))
}

/// Bulk document ingestion endpoint
#[utoipa::path(
    post,
    path = "/api/documents/bulk",
    tag = "documents",
    request_body = BulkIngestRequest,
    responses(
        (status = 201, description = "Documents ingested", body = BulkIngestResponse)
    )
)]
pub async fn bulk_ingest_documents_handler(
    state: web::Data<Arc<AppState>>,
    req: web::Json<BulkIngestRequest>,
) -> Result<HttpResponse, ApiError> {
    let mut document_ids = Vec::with_capacity(req.documents.len());
    let mut total_chunks = 0;

    for ingest_req in &req.documents {
        let doc = types::Document {
            id: types::DocumentId::new(uuid::Uuid::new_v4().to_string()),
            title: ingest_req.title.clone(),
            author: ingest_req.author.clone(),
            date: ingest_req.date.clone(),
            source_collection: ingest_req.source_collection.clone(),
            source_url: ingest_req.source_url.clone(),
            document_type: ingest_req.document_type.clone(),
            text: ingest_req.text.clone(),
            metadata: std::collections::HashMap::new(),
        };

        let doc_id = doc.id.to_string();
        let chunks_added = state.ingest_document(doc)?;
        document_ids.push(doc_id);
        total_chunks += chunks_added;
    }

    Ok(HttpResponse::Created().json(BulkIngestResponse {
        document_ids,
        total_chunks,
    }))
}

/// Get document endpoint
#[utoipa::path(
    get,
    path = "/api/documents/{id}",
    tag = "documents",
    params(
        ("id" = String, Path, description = "Document ID")
    ),
    responses(
        (status = 200, description = "Document retrieved", body = Object),
        (status = 404, description = "Document not found")
    )
)]
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
#[utoipa::path(
    delete,
    path = "/api/documents/{id}",
    tag = "documents",
    params(
        ("id" = String, Path, description = "Document ID")
    ),
    responses(
        (status = 204, description = "Document deleted")
    )
)]
pub async fn delete_document_handler(path: web::Path<String>) -> HttpResponse {
    let _doc_id = path.into_inner();
    // TODO: Implement document deletion
    HttpResponse::NoContent().finish()
}

/// Get index statistics endpoint
#[utoipa::path(
    get,
    path = "/api/index/stats",
    tag = "system",
    responses(
        (status = 200, description = "Index statistics", body = Object)
    )
)]
pub async fn get_index_stats_handler(
    state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    let stats = state.get_stats();
    HttpResponse::Ok().json(stats)
}

/// Export index endpoint
#[utoipa::path(
    get,
    path = "/api/index/export",
    tag = "system",
    responses(
        (status = 200, description = "Index exported", body = Object)
    )
)]
pub async fn export_index_handler(
    state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    let exported = state.export_json();
    HttpResponse::Ok().json(exported)
}
