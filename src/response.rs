use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

/// Standard API response envelope.
///
/// All endpoints return one of two shapes:
///
/// **Success** (2xx):
/// ```json
/// { "data": <payload> }
/// ```
///
/// **Error** (4xx/5xx):
/// ```json
/// { "error": { "code": "NOT_FOUND", "message": "Resource not found" } }
/// ```

// ── Success helpers ──

/// `200 OK` with a data payload.
pub fn ok<T: serde::Serialize>(data: T) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "data": data })))
}

/// `201 Created` with a data payload.
pub fn created<T: serde::Serialize>(data: T) -> (StatusCode, Json<Value>) {
    (StatusCode::CREATED, Json(json!({ "data": data })))
}

// ── Unified application error ──

#[derive(Debug)]
pub enum AppError {
    #[allow(dead_code)]
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    Internal(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            AppError::Internal(err) => {
                tracing::error!(?err, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "an internal error occurred".to_string(),
                )
            }
        };

        (
            status,
            Json(json!({
                "error": {
                    "code": code,
                    "message": message
                }
            })),
        )
            .into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Internal(err.into())
    }
}
