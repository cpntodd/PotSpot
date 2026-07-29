use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};

use crate::auth::middleware::AuthUser;
use crate::errors::{AppError, AppResult};
use crate::models::PrivateStrainRequest;
use crate::services::vault_service;
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

/// GET /api/v1/vault
async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    let vault = vault_service::list_vault(&state.pool, auth.user_id).await?;
    Ok(Json(vault))
}

/// POST /api/v1/vault
async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<PrivateStrainRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let id = vault_service::create_private_strain(&state.pool, auth.user_id, &req).await?;
    Ok(Json(serde_json::json!({
        "message": "Private strain created",
        "id": id,
    })))
}

/// GET /api/v1/vault/:id
async fn detail(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let strain = vault_service::get_private_strain(&state.pool, auth.user_id, id).await?;
    Ok(Json(serde_json::to_value(strain).map_err(|e| AppError::Internal(e.into()))?))
}

/// PUT /api/v1/vault/:id
async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<PrivateStrainRequest>,
) -> AppResult<Json<serde_json::Value>> {
    vault_service::update_private_strain(&state.pool, auth.user_id, id, &req).await?;
    Ok(Json(serde_json::json!({ "message": "Private strain updated" })))
}

/// DELETE /api/v1/vault/:id
async fn remove(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    vault_service::delete_private_strain(&state.pool, auth.user_id, id).await?;
    Ok(Json(serde_json::json!({ "message": "Private strain deleted" })))
}

/// POST /api/v1/vault/:id/push
async fn push_to_public(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let public_id = vault_service::push_to_public(&state.pool, auth.user_id, id).await?;
    Ok(Json(serde_json::json!({
        "message": "Strain pushed to public catalog. It will be reviewed by vetters.",
        "public_strain_id": public_id,
    })))
}

/// POST /api/v1/vault/:id/push-update
async fn push_update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    let summary = body.get("change_summary").and_then(|v| v.as_str());
    vault_service::push_update_to_public(&state.pool, auth.user_id, id, summary).await?;
    Ok(Json(serde_json::json!({
        "message": "Updates pushed to public strain. Changes are pending vetting review.",
    })))
}

/// POST /api/v1/vault/save/:strain_id
async fn save_public(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(strain_id): Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    vault_service::save_public_strain(&state.pool, auth.user_id, strain_id).await?;
    Ok(Json(serde_json::json!({ "message": "Strain saved to vault" })))
}

/// DELETE /api/v1/vault/save/:strain_id
async fn unsave_public(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(strain_id): Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    vault_service::unsave_public_strain(&state.pool, auth.user_id, strain_id).await?;
    Ok(Json(serde_json::json!({ "message": "Strain removed from vault" })))
}
