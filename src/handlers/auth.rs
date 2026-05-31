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

/// Return the currently authenticated user's info from the JWT payload.
pub async fn get_me(client: NeonClient) -> Result<impl axum::response::IntoResponse, AppError> {
    let token = client
        .token()
        .ok_or_else(|| AppError::Unauthorized("not authenticated".into()))?;
    let user = decode_jwt_payload(token)
        .map_err(|e| AppError::Unauthorized(format!("invalid token: {}", e)))?;
    Ok(response::ok(user))
}

/// Decode the payload segment of a JWT without verifying the signature.
/// We only extract what the auth service already signed — trust is inherited
/// from the `set-auth-jwt` response header.
fn decode_jwt_payload(token: &str) -> Result<serde_json::Value, String> {
    use base64::Engine;

    let payload_b64 = token
        .split('.')
        .nth(1)
        .ok_or_else(|| "missing payload segment".to_string())?;

    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let bytes = engine
        .decode(payload_b64)
        .map_err(|e| format!("base64 decode failed: {}", e))?;

    serde_json::from_slice(&bytes).map_err(|e| format!("json decode failed: {}", e))
}

fn map_auth_error(err: anyhow::Error) -> AppError {
    let msg = err.to_string();
    let status_code = msg
        .split(": ")
        .nth(1)
        .and_then(|s| s.split(' ').next())
        .and_then(|s| s.parse::<u16>().ok());
    let body = msg.split(" - ").nth(1).unwrap_or(&msg).to_string();
    let is_user_exists = body.contains("already exists");

    match (status_code, is_user_exists) {
        (Some(400), _) | (Some(422), _) | (_, true) => AppError::BadRequest(body),
        (Some(401), false) => AppError::Unauthorized(body),
        _ => AppError::Internal(err),
    }
}
