use std::sync::Arc;

use axum::{Json, extract::State};

use crate::config::Config;
use crate::config::client::{NeonClient, SignInRequest, SignUpRequest};
use crate::response::{self, AppError};

pub async fn sign_up(
    State(config): State<Arc<Config>>,
    Json(body): Json<SignUpRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let mut client = NeonClient::new(&config);
    match client.sign_up(body.email, body.name, body.password).await {
        Ok(token) => Ok(response::created(serde_json::json!({ "token": token }))),
        Err(err) => Err(map_auth_error(err)),
    }
}

pub async fn sign_in(
    State(config): State<Arc<Config>>,
    Json(body): Json<SignInRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let mut client = NeonClient::new(&config);
    match client.sign_in(body.email, body.password).await {
        Ok(token) => Ok(response::ok(serde_json::json!({ "token": token }))),
        Err(err) => Err(map_auth_error(err)),
    }
}

pub async fn sign_out(
    State(config): State<Arc<Config>>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let mut client = NeonClient::new(&config);
    client.sign_out().await?;
    Ok(response::ok(serde_json::json!({ "message": "signed out" })))
}

/// Map an anyhow error from the auth client to an `AppError` with the
/// right status code, preserving the API error message.
fn map_auth_error(err: anyhow::Error) -> AppError {
    let msg = err.to_string();
    let status_code = msg
        .split(": ")
        .nth(1)
        .and_then(|s| s.split(' ').next())
        .and_then(|s| s.parse::<u16>().ok());
    let body = msg.split(" - ").nth(1).unwrap_or(&msg).to_string();

    // Override API status code based on the error message content.
    let is_user_exists = body.contains("already exists");

    match (status_code, is_user_exists) {
        (Some(400), _) | (Some(422), _) | (_, true) => AppError::BadRequest(body),
        (Some(401), false) => AppError::Unauthorized(body),
        _ => AppError::Internal(err),
    }
}
