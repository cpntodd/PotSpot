use axum::{extract::State, routing::{get, post, put, delete}, Json, Router};

use crate::errors::AppResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/", post(create))
        .route("/{id}", get(detail))
        .route("/{id}", put(update))
        .route("/{id}", delete(remove))
        .route("/{id}/push", post(push_to_public))
        .route("/{id}/push-update", post(push_update))
        .route("/save/{strain_id}", post(save_public))
        .route("/save/{strain_id}", delete(unsave_public))
}

async fn list(
    State(_state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: List user's private strains + saved strains
    Ok(Json(serde_json::json!({ "strains": [], "saved": [] })))
}

async fn create(
    State(_state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: Create private strain
    Ok(Json(serde_json::json!({ "message": "Not yet implemented" })))
}

async fn detail(
    State(_state): State<AppState>,
    axum::extract::Path(_id): axum::extract::Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "message": "Not yet implemented" })))
}

async fn update(
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

async fn push_to_public(
    State(_state): State<AppState>,
    axum::extract::Path(_id): axum::extract::Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: Push private strain to public catalog
    Ok(Json(serde_json::json!({ "message": "Not yet implemented" })))
}

async fn push_update(
    State(_state): State<AppState>,
    axum::extract::Path(_id): axum::extract::Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "message": "Not yet implemented" })))
}

async fn save_public(
    State(_state): State<AppState>,
    axum::extract::Path(_strain_id): axum::extract::Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "message": "Not yet implemented" })))
}

async fn unsave_public(
    State(_state): State<AppState>,
    axum::extract::Path(_strain_id): axum::extract::Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "message": "Not yet implemented" })))
}
