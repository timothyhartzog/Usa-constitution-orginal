//! Constitutional Research System - Web Service
//!
//! HTTP API for document ingestion, indexing, and semantic search
//! with WebSocket support for live updates.

use actix_web::{web, App, HttpServer, middleware};
use log::info;

mod handlers;
mod state;
mod error_response;
mod db;
pub mod ws;
mod auth;

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web_httpauth::middleware::HttpAuthentication;

use state::AppState;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use handlers::ApiDoc;

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
        let openapi = ApiDoc::openapi();

        // Rate limiting configuration (e.g., 5 requests per second)
        let governor_conf = GovernorConfigBuilder::default()
            .per_millisecond(200)
            .burst_size(10)
            .finish()
            .unwrap();

        // Auth middleware
        let auth = HttpAuthentication::bearer(auth::validator);

        App::new()
            .app_data(web::JsonConfig::default().limit(50 * 1024 * 1024))
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
            // Swagger UI
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
            // Public endpoints
            .route("/health", web::get().to(handlers::health_handler))
            .route("/ws", web::get().to(ws::ws_handler))
            // Protected API scope
            .service(
                web::scope("/api")
                    .wrap(auth)
                    .wrap(Governor::new(&governor_conf))
                    // Search endpoints
                    .route("/search", web::post().to(handlers::search_handler))
                    .route("/search/fulltext", web::post().to(handlers::search_fulltext_handler))
                    .route("/search/fuzzy", web::post().to(handlers::search_fuzzy_handler))
                    .route("/search/semantic", web::post().to(handlers::search_semantic_handler))
                    // Document management
                    .route("/documents/bulk", web::post().to(handlers::bulk_ingest_documents_handler))
                    .route("/documents", web::post().to(handlers::ingest_document_handler))
                    .route("/documents/{id}", web::get().to(handlers::get_document_handler))
                    .route("/documents/{id}", web::delete().to(handlers::delete_document_handler))
                    // Index management
                    .route("/index", web::get().to(handlers::get_index_stats_handler))
                    .route("/index/export", web::get().to(handlers::export_index_handler))
            )
    })
    .bind("127.0.0.1:8082")?
    .run()
    .await
}
