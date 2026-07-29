use axum::{extract::State, routing::get, Json, Router};

use crate::errors::AppResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/stats", get(stats))
}

async fn list_users(
    State(_state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "users": [] })))
}

async fn stats(
    State(_state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "user_count": 0,
        "strain_count": 0,
        "comment_count": 0,
    })))
}
