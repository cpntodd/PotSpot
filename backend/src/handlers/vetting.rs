use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::auth::middleware::AuthUser;
use crate::errors::{AppError, AppResult};
use crate::services::{notification_service, vetting_service};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/queue", get(queue))
        .route("/{revision_id}/approve", post(approve))
        .route("/{revision_id}/reject", post(reject))
}

/// Request body for rejecting a revision.
#[derive(Debug, Deserialize)]
struct RejectBody {
    reason: String,
}

/// GET /api/v1/vetting/queue
///
/// List all pending revisions. Vetter+ role required.
async fn queue(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_vetter()?;
    let revisions = vetting_service::get_pending_queue(&state.pool).await?;
    Ok(Json(serde_json::to_value(revisions).map_err(|e| AppError::Internal(e.into()))?))
}

/// POST /api/v1/vetting/:revision_id/approve
///
/// Approve a pending revision. Vetter+ role required.
async fn approve(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(revision_id): Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_vetter()?;
    vetting_service::approve_revision(&state.pool, revision_id, auth.user_id).await?;

    // Notify the revision proposer
    let proposer: Option<uuid::Uuid> = sqlx::query_scalar(
        "SELECT proposed_by FROM strain_revisions WHERE id = $1",
    )
    .bind(revision_id)
    .fetch_optional(&state.pool)
    .await?;

    if let Some(user_id) = proposer {
        let _ = notification_service::create_notification(
            &state.pool,
            user_id,
            "strain_approved",
            Some(revision_id),
            "Your strain edit was approved",
        )
        .await;
    }

    Ok(Json(serde_json::json!({ "message": "Revision approved" })))
}

/// POST /api/v1/vetting/:revision_id/reject
///
/// Reject a pending revision. Rolls back the strain to its previous state.
/// Vetter+ role required.
async fn reject(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(revision_id): Path<uuid::Uuid>,
    Json(req): Json<RejectBody>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_vetter()?;
    if req.reason.trim().is_empty() {
        return Err(AppError::BadRequest("Rejection reason is required".into()));
    }

    vetting_service::reject_revision(&state.pool, revision_id, auth.user_id, &req.reason).await?;

    // Notify the revision proposer
    let proposer: Option<uuid::Uuid> = sqlx::query_scalar(
        "SELECT proposed_by FROM strain_revisions WHERE id = $1",
    )
    .bind(revision_id)
    .fetch_optional(&state.pool)
    .await?;

    if let Some(user_id) = proposer {
        let _ = notification_service::create_notification(
            &state.pool,
            user_id,
            "strain_rejected",
            Some(revision_id),
            &format!("Your strain edit was rejected: {}", req.reason),
        )
        .await;
    }

    Ok(Json(serde_json::json!({ "message": "Revision rejected and strain rolled back" })))
}
