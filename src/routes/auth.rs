use axum::{Json, extract::State, routing::post};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::config::Config;
use crate::error::AppError;
use crate::services::auth::NeonAuthClient;

pub fn routes() -> axum::Router<Arc<Config>> {
    axum::Router::new()
        .route("/sign-up", post(sign_up))
        .route("/sign-in", post(sign_in))
        .route("/sign-out", post(sign_out))
        .route("/session", post(get_session))
}

// ── Request bodies ──

#[derive(Deserialize)]
pub struct SignUpBody {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignInBody {
    pub email: String,
    pub password: String,
}

// ── Handlers ──

/// POST /api/v1/auth/sign-up
async fn sign_up(
    State(config): State<Arc<Config>>,
    Json(body): Json<SignUpBody>,
) -> Result<Json<Value>, AppError> {
    let mut client = NeonAuthClient::new(&config);
    let response = client.sign_up(body.email, body.name, body.password).await?;

    Ok(Json(json!({
        "token": response.session.token
    })))
}

/// POST /api/v1/auth/sign-in
async fn sign_in(
    State(config): State<Arc<Config>>,
    Json(body): Json<SignInBody>,
) -> Result<Json<Value>, AppError> {
    let mut client = NeonAuthClient::new(&config);
    let response = client.sign_in(body.email, body.password).await?;

    Ok(Json(json!({
        "token": response.session.token
    })))
}

/// POST /api/v1/auth/sign-out
async fn sign_out(
    State(config): State<Arc<Config>>,
    // In a real app, extract the Bearer token from the Authorization header
    // and pass it to `NeonAuthClient::with_token`.
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Value>, AppError> {
    let _body = body; // placeholder – token extraction goes here
    let mut client = NeonAuthClient::new(&config);
    client.sign_out().await?;

    Ok(Json(json!({ "message": "signed out" })))
}

/// POST /api/v1/auth/session
async fn get_session(
    State(config): State<Arc<Config>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Value>, AppError> {
    let _body = body; // placeholder – token extraction goes here
    let mut client = NeonAuthClient::new(&config);
    let session = client.get_session().await?;

    Ok(Json(json!({ "session": session })))
}
