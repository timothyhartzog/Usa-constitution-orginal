//! Constitutional Research System - Web Service
//!
//! HTTP API for document ingestion, indexing, and semantic search
//! with WebSocket support for live updates.

use actix_web::{web, App, HttpServer, middleware};
use constitutional_lib::{chunker, error, fulltext_index, fuzzy_match, metadata_tagger, tokenizer, types, vector_store};
use log::info;
use std::sync::Arc;

mod handlers;
mod state;
mod error_response;
mod db;

use state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .try_init()
        .ok();

    info!("Starting Constitutional Research System API Server");

    // Initialize app state with optional database persistence
    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "constitution.db".to_string());
    let app_state = match std::fs::metadata(&db_path) {
        Ok(_) => {
            // Database exists, recover from it
            match AppState::with_db(&db_path) {
                Ok(state) => {
                    info!("Recovered indexes from existing database: {}", db_path);
                    std::sync::Arc::new(state)
                }
                Err(e) => {
                    log::warn!("Failed to recover from database: {}, starting fresh", e);
                    std::sync::Arc::new(AppState::new())
                }
            }
        }
        Err(_) => {
            // New database, will be created on first insert
            info!("Creating new database: {}", db_path);
            match AppState::with_db(&db_path) {
                Ok(state) => std::sync::Arc::new(state),
                Err(e) => {
                    log::warn!("Failed to initialize database: {}, using in-memory mode", e);
                    std::sync::Arc::new(AppState::new())
                }
            }
        }
    };

    info!("Listening on http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
            // Search endpoints
            .route("/api/search", web::post().to(handlers::search_handler))
            .route("/api/search/fulltext", web::post().to(handlers::search_fulltext_handler))
            .route("/api/search/fuzzy", web::post().to(handlers::search_fuzzy_handler))
            .route("/api/search/semantic", web::post().to(handlers::search_semantic_handler))
            // Document management
            .route("/api/documents", web::post().to(handlers::ingest_document_handler))
            .route("/api/documents/{id}", web::get().to(handlers::get_document_handler))
            .route("/api/documents/{id}", web::delete().to(handlers::delete_document_handler))
            // Index management
            .route("/api/index", web::get().to(handlers::get_index_stats_handler))
            .route("/api/index/export", web::get().to(handlers::export_index_handler))
            // Health check
            .route("/health", web::get().to(handlers::health_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
