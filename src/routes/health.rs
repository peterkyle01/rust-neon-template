use axum::Json;
use serde_json::{Value, json};

/// Simple health check endpoint.
pub async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
