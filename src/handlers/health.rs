use crate::response;
use crate::response::AppError;

pub async fn health_check() -> Result<impl axum::response::IntoResponse, AppError> {
    Ok(response::ok(serde_json::json!({ "status": "ok" })))
}
