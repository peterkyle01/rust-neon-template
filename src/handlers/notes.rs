use axum::{Json, extract::Path};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::client::NeonClient;
use crate::response::{self, AppError};
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
) -> Result<impl axum::response::IntoResponse, AppError> {
    let notes: Vec<Note> = client.create("notes", &body).await?;
    let note = notes
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound("no note returned".into()))?;
    Ok(response::created(json!(note)))
}

pub async fn get_my_notes(
    client: NeonClient,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let notes: Vec<Note> = client.get_all("notes").await?;
    Ok(response::ok(json!(notes)))
}

pub async fn get_note(
    client: NeonClient,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let note: Option<Note> = client.get_one("notes", id).await?;
    match note {
        Some(n) => Ok(response::ok(json!(n))),
        None => Err(AppError::NotFound(format!("note {} not found", id))),
    }
}

pub async fn update_note(
    client: NeonClient,
    Path(id): Path<i32>,
    Json(body): Json<RequestNote>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let notes: Vec<Note> = client.update("notes", id, &body).await?;
    if notes.is_empty() {
        return Err(AppError::NotFound(format!("note {} not found", id)));
    }
    Ok(response::ok(json!(notes)))
}

pub async fn delete_note(
    client: NeonClient,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let deleted = client.delete("notes", id).await?;
    if !deleted {
        return Err(AppError::NotFound(format!("note {} not found", id)));
    }
    Ok(response::ok(json!({ "message": "deleted" })))
}
