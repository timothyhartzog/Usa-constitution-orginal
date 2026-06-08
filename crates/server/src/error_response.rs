//! HTTP error response handling

use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use std::fmt;

/// API error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

/// Application error type
#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    InvalidRequest(String),
    InternalError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "{}", msg),
            ApiError::InvalidRequest(msg) => write!(f, "{}", msg),
            ApiError::InternalError(msg) => write!(f, "{}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let (status, code, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            ApiError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, "INVALID_REQUEST", msg.clone()),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone()),
        };

        HttpResponse::build(status).json(ErrorResponse {
            error: message,
            code: code.to_string(),
        })
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<constitutional_lib::Error> for ApiError {
    fn from(err: constitutional_lib::Error) -> Self {
        ApiError::InternalError(err.to_string())
    }
}
