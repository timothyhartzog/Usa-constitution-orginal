use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct SearchFilters {
    pub document_id: Option<String>,
    pub author: Option<String>,
    pub issue_tag: Option<String>,
    pub clause_tag: Option<String>,
    pub min_date: Option<String>,
    pub max_date: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub search_type: Option<String>,
    pub max_results: Option<usize>,
    pub filters: Option<SearchFilters>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub chunk_id: String,
    pub document_id: String,
    pub document_title: String,
    pub document_author: Option<String>,
    pub chunk_title: Option<String>,
    pub preview: String,
    pub score: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub count: usize,
    pub search_type: String,
}

pub async fn perform_search(request: &SearchRequest) -> Result<SearchResponse, String> {
    // In dev, the API is running at :8080, in prod it might be same host
    // Adding Authorization token from dev environment
    let token = "secret-token"; 

    let response = gloo_net::http::Request::post("http://127.0.0.1:8082/api/search")
        .header("Authorization", &format!("Bearer {}", token))
        .json(request)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response.json::<SearchResponse>().await.map_err(|e| e.to_string())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexStatsResponse {
    pub num_documents: usize,
    pub num_chunks: usize,
    pub vector_dimensions: Option<usize>,
    pub unique_terms: usize,
    pub memory_usage_bytes: usize,
}

pub async fn get_index_stats() -> Result<IndexStatsResponse, String> {
    let token = "secret-token"; 

    let response = gloo_net::http::Request::get("http://127.0.0.1:8082/api/index/stats")
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response.json::<IndexStatsResponse>().await.map_err(|e| e.to_string())
}

pub async fn export_index() -> Result<serde_json::Value, String> {
    let token = "secret-token"; 

    let response = gloo_net::http::Request::get("http://127.0.0.1:8082/api/index/export")
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response.json::<serde_json::Value>().await.map_err(|e| e.to_string())
}
