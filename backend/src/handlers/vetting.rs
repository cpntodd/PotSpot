use axum::{extract::State, routing::get, Json, Router};

use crate::errors::AppResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/queue", get(queue))
        .route("/{revision_id}/approve", axum::routing::post(approve))
        .route("/{revision_id}/reject", axum::routing::post(reject))
}

async fn queue(
    State(_state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "revisions": [] })))
}

async fn approve(
    State(_state): State<AppState>,
    axum::extract::Path(_revision_id): axum::extract::Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "message": "Not yet implemented" })))
}

async fn reject(
    State(_state): State<AppState>,
    axum::extract::Path(_revision_id): axum::extract::Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "message": "Not yet implemented" })))
}
