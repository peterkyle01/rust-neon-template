use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// Unified application error type.
///
/// Converts to HTTP responses with appropriate status codes and JSON bodies.
#[derive(Debug)]
pub enum AppError {
    /// Invalid request from the client.
    BadRequest(String),
    /// Authentication required or failed.
    Unauthorized(String),
    /// Resource not found.
    NotFound(String),
    /// Internal server error (details are logged, not leaked to client).
    Internal(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(err) => {
                // Log the internal error, return a generic message to the client.
                tracing::error!(?err, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "an internal error occurred".to_string(),
                )
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

/// Convenience for converting `anyhow::Error` into `AppError::Internal`.
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err)
    }
}

/// Convenience for converting `reqwest::Error` into `AppError::Internal`.
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Internal(err.into())
    }
}
