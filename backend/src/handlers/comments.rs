use axum::{extract::State, routing::{get, post, put, delete}, Json, Router};

use crate::errors::AppResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{id}/reply", post(reply))
        .route("/{id}", put(edit))
        .route("/{id}", delete(remove))
        .route("/{id}/vote", post(vote))
        .route("/{id}/vote", delete(remove_vote))
}

async fn reply(
    State(_state): State<AppState>,
    axum::extract::Path(_id): axum::extract::Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "message": "Not yet implemented" })))
}

async fn edit(
    State(_state): State<AppState>,
    axum::extract::Path(_id): axum::extract::Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "message": "Not yet implemented" })))
}

async fn remove(
    State(_state): State<AppState>,
    axum::extract::Path(_id): axum::extract::Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "message": "Not yet implemented" })))
}

async fn vote(
    State(_state): State<AppState>,
    axum::extract::Path(_id): axum::extract::Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "message": "Not yet implemented" })))
}

async fn remove_vote(
    State(_state): State<AppState>,
    axum::extract::Path(_id): axum::extract::Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "message": "Not yet implemented" })))
}
