use std::sync::Arc;

use axum::{Json, extract::State};
use serde_json::{Value, json};

use crate::config::Config;
use crate::config::client::{NeonClient, SignInRequest, SignUpRequest};
use crate::error::AppError;

pub async fn sign_up(
    State(config): State<Arc<Config>>,
    Json(body): Json<SignUpRequest>,
) -> Result<Json<Value>, AppError> {
    let mut client = NeonClient::new(&config);
    let token = client.sign_up(body.email, body.name, body.password).await?;
    Ok(Json(json!({ "token": token })))
}

pub async fn sign_in(
    State(config): State<Arc<Config>>,
    Json(body): Json<SignInRequest>,
) -> Result<Json<Value>, AppError> {
    let mut client = NeonClient::new(&config);
    let token = client.sign_in(body.email, body.password).await?;
    Ok(Json(json!({ "token": token })))
}

pub async fn sign_out(State(config): State<Arc<Config>>) -> Result<Json<Value>, AppError> {
    let mut client = NeonClient::new(&config);
    client.sign_out().await?;
    Ok(Json(json!({ "message": "signed out" })))
}

pub async fn get_session(State(config): State<Arc<Config>>) -> Result<Json<Value>, AppError> {
    let mut client = NeonClient::new(&config);
    let session = client.get_session().await?;
    Ok(Json(json!({ "session": session })))
}
