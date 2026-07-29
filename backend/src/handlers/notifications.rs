use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::auth::middleware::AuthUser;
use crate::errors::AppResult;
use crate::services::notification_service;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/unread-count", get(unread_count))
        .route("/{id}/read", post(mark_read))
        .route("/read-all", post(mark_all_read))
        .route("/preferences", post(update_preferences))
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    page: Option<i64>,
    per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct PreferencesBody {
    notification_type: String,
    enabled: bool,
}

async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(100);
    let (notifications, total) = notification_service::get_notifications(&state.pool, auth.user_id, page, per_page).await?;
    Ok(Json(serde_json::json!({
        "notifications": notifications,
        "total": total,
        "page": page,
        "per_page": per_page,
    })))
}

async fn unread_count(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    let count = notification_service::get_unread_count(&state.pool, auth.user_id).await?;
    Ok(Json(serde_json::json!({ "unread_count": count })))
}

async fn mark_read(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    notification_service::mark_read(&state.pool, auth.user_id, id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

async fn mark_all_read(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    notification_service::mark_all_read(&state.pool, auth.user_id).await?;
    Ok(Json(serde_json::json!({ "message": "All notifications marked as read" })))
}

async fn update_preferences(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<PreferencesBody>,
) -> AppResult<Json<serde_json::Value>> {
    notification_service::update_preferences(&state.pool, auth.user_id, &req.notification_type, req.enabled).await?;
    Ok(Json(serde_json::json!({ "message": "Preferences updated" })))
}
