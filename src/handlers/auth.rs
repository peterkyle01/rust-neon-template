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
    let token = client.sign_up(body.email, body.name, body.password).await?;
    Ok(response::created(serde_json::json!({ "token": token })))
}

pub async fn sign_in(
    State(config): State<Arc<Config>>,
    Json(body): Json<SignInRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let mut client = NeonClient::new(&config);
    let token = client.sign_in(body.email, body.password).await?;
    Ok(response::ok(serde_json::json!({ "token": token })))
}

pub async fn sign_out(
    State(config): State<Arc<Config>>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let mut client = NeonClient::new(&config);
    client.sign_out().await?;
    Ok(response::ok(serde_json::json!({ "message": "signed out" })))
}
