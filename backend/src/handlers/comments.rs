use axum::{
    extract::{Path, State},
    routing::{delete, post, put},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::errors::{AppError, AppResult};
use crate::services::{comment_service, notification_service};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{id}/reply", post(reply))
        .route("/{id}", put(edit))
        .route("/{id}", delete(remove))
        .route("/{id}/vote", post(vote))
        .route("/{id}/vote", delete(remove_vote))
}

/// Request body for posting/editing a comment.
#[derive(Debug, Deserialize)]
struct CommentBody {
    body: String,
}

/// Request body for voting.
#[derive(Debug, Deserialize)]
struct VoteBody {
    vote: i16,
}

/// POST /api/v1/comments/:id/reply
async fn reply(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(parent_id): Path<Uuid>,
    Json(req): Json<CommentBody>,
) -> AppResult<Json<serde_json::Value>> {
    // Get the parent comment to find the strain_id
    let strain_id: Uuid = sqlx::query_scalar("SELECT strain_id FROM comments WHERE id = $1")
        .bind(parent_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Parent comment not found".into()))?;

    let id = comment_service::post_comment(
        &state.pool,
        strain_id,
        auth.user_id,
        Some(parent_id),
        &req.body,
    )
    .await?;

    // Notify the parent comment author (unless they're replying to themselves)
    let parent_author: Option<uuid::Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM comments WHERE id = $1",
    )
    .bind(parent_id)
    .fetch_optional(&state.pool)
    .await?;

    if let Some(author_id) = parent_author {
        if author_id != auth.user_id {
            let _ = notification_service::create_notification(
                &state.pool,
                author_id,
                "comment_reply",
                Some(id),
                &format!("{} replied to your comment", ""), // TODO: fetch display_name
            )
            .await;
        }
    }

    Ok(Json(serde_json::json!({
        "message": "Reply posted",
        "id": id,
    })))
}

/// PUT /api/v1/comments/:id
async fn edit(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CommentBody>,
) -> AppResult<Json<serde_json::Value>> {
    comment_service::edit_comment(&state.pool, auth.user_id, id, &req.body).await?;
    Ok(Json(serde_json::json!({ "message": "Comment updated" })))
}

/// DELETE /api/v1/comments/:id
async fn remove(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    comment_service::delete_comment(&state.pool, auth.user_id, id).await?;
    Ok(Json(serde_json::json!({ "message": "Comment deleted" })))
}

/// POST /api/v1/comments/:id/vote
async fn vote(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<VoteBody>,
) -> AppResult<Json<serde_json::Value>> {
    comment_service::vote_comment(&state.pool, auth.user_id, id, req.vote).await?;
    Ok(Json(serde_json::json!({ "message": "Vote recorded" })))
}

/// DELETE /api/v1/comments/:id/vote
async fn remove_vote(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    comment_service::remove_vote(&state.pool, auth.user_id, id).await?;
    Ok(Json(serde_json::json!({ "message": "Vote removed" })))
}
