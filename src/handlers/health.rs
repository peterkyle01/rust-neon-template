use std::sync::Arc;

use axum::extract::State;

use crate::config::Config;
use crate::response::{self, AppError};

pub async fn health_check(
    State(config): State<Arc<Config>>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let mut auth_ok = false;
    let mut data_api_ok = false;

    // Check Neon Auth — fetch the JWKS endpoint.
    match reqwest::get(format!("{}/.well-known/jwks.json", config.auth_url)).await {
        Ok(resp) => auth_ok = resp.status().is_success(),
        Err(_) => {}
    }

    // Check Neon Data API — a lightweight GET to see if it responds.
    match reqwest::Client::new()
        .get(&config.data_api_url)
        .send()
        .await
    {
        Ok(resp) => data_api_ok = !resp.status().is_server_error(),
        Err(_) => {}
    }

    let overall = auth_ok && data_api_ok;

    Ok(response::ok(serde_json::json!({
        "status": if overall { "ok" } else { "degraded" },
        "checks": {
            "auth": if auth_ok { "ok" } else { "unreachable" },
            "data_api": if data_api_ok { "ok" } else { "unreachable" }
        }
    })))
}
