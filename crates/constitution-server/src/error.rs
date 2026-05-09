//! API error type with a stable JSON shape.
//!
//! Any handler returning `Result<T, ApiError>` will yield a JSON body of
//! the form `{ "error": "...", "kind": "not_found" }` with the appropriate
//! HTTP status code.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use constitution_archive::ArchiveError;
use serde::Serialize;

/// Errors returned to API clients.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// 404 — chunk / event / target not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// 400 — malformed query / unparseable phase / etc.
    #[error("bad request: {0}")]
    BadRequest(String),

    /// 500 — anything else.
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<ArchiveError> for ApiError {
    fn from(e: ArchiveError) -> Self {
        match e {
            ArchiveError::ChunkNotFound(id) => Self::NotFound(format!("chunk {id}")),
            ArchiveError::ProcessEventNotFound(id) => Self::NotFound(format!("event {id}")),
            other => Self::Internal(other.to_string()),
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    kind: &'static str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, kind) = match &self {
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        };
        let body = ErrorBody {
            error: self.to_string(),
            kind,
        };
        (status, Json(body)).into_response()
    }
}
