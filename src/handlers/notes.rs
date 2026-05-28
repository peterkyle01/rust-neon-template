use axum::{Json, extract::Path};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::config::client::NeonClient;
use crate::error::AppError;
use utility_types::Omit;

// ── Note model ──

#[derive(Debug, Clone, Serialize, Deserialize, Omit)]
#[omit(arg(ident=RequestNote, fields(id), derive(Serialize, Deserialize)))]
pub struct Note {
    pub id: i32,
    pub title: String,
    pub content: String,
}

// ── Handlers ──

pub async fn create_note(
    client: NeonClient,
    Json(body): Json<RequestNote>,
) -> Result<Json<Value>, AppError> {
    let notes: Vec<Note> = client.create("notes", &body).await?;
    let note = notes
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound("no note returned".into()))?;
    Ok(Json(json!(note)))
}

pub async fn get_my_notes(client: NeonClient) -> Result<Json<Value>, AppError> {
    let notes: Vec<Note> = client.get_all("notes").await?;
    Ok(Json(json!(notes)))
}

pub async fn get_note(client: NeonClient, Path(id): Path<i32>) -> Result<Json<Value>, AppError> {
    let note: Option<Note> = client.get_one("notes", id).await?;
    Ok(Json(json!(note)))
}

pub async fn update_note(
    client: NeonClient,
    Path(id): Path<i32>,
    Json(body): Json<RequestNote>,
) -> Result<Json<Value>, AppError> {
    let notes: Vec<Note> = client.update("notes", id, &body).await?;
    Ok(Json(json!(notes)))
}

pub async fn delete_note(client: NeonClient, Path(id): Path<i32>) -> Result<Json<Value>, AppError> {
    client.delete("notes", id).await?;
    Ok(Json(json!({ "message": "deleted" })))
}
