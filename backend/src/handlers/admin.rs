use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;

use crate::auth::middleware::AuthUser;
use crate::errors::{AppError, AppResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/{id}/role", put(set_role))
        .route("/stats", get(stats))
}

#[derive(Debug, Deserialize)]
struct SetRoleBody {
    role: String,
}

/// GET /api/v1/admin/users
async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_admin()?;
    let users = sqlx::query_as::<_, (uuid::Uuid, String, String, String, bool, chrono::DateTime<chrono::Utc>)>(
        r#"SELECT id, email, display_name, role::text, age_verified, created_at
           FROM users WHERE deleted_at IS NULL ORDER BY created_at DESC"#,
    )
    .fetch_all(&state.pool)
    .await?;

    let user_list: Vec<serde_json::Value> = users
        .into_iter()
        .map(|(id, email, display_name, role, age_verified, created_at)| {
            serde_json::json!({
                "id": id,
                "email": email,
                "display_name": display_name,
                "role": role,
                "age_verified": age_verified,
                "created_at": created_at,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "users": user_list })))
}

/// PUT /api/v1/admin/users/:id/role
///
/// Change a user's role. Admin only.
async fn set_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(user_id): Path<uuid::Uuid>,
    Json(req): Json<SetRoleBody>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_admin()?;
    // Validate role
    let valid_roles = ["user", "vetter", "admin"];
    if !valid_roles.contains(&req.role.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid role '{}'. Must be one of: {}",
            req.role,
            valid_roles.join(", ")
        )));
    }

    let rows = sqlx::query("UPDATE users SET role = $1::user_role WHERE id = $2")
        .bind(&req.role)
        .bind(user_id)
        .execute(&state.pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound("User not found".into()));
    }

    // Log to audit log
    sqlx::query(
        r#"INSERT INTO admin_audit_log (admin_id, action, target_id, details)
           VALUES ($1, 'user_role_change', $2, $3)"#,
    )
    .bind(auth.user_id)
    .bind(user_id)
    .bind(serde_json::json!({ "new_role": req.role }))
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "message": format!("User role updated to {}", req.role) })))
}

/// GET /api/v1/admin/stats
async fn stats(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_admin()?;
    let user_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE deleted_at IS NULL",
    )
    .fetch_one(&state.pool)
    .await?;

    let strain_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public_strains WHERE is_active = true",
    )
    .fetch_one(&state.pool)
    .await?;

    let comment_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM comments WHERE is_deleted = false",
    )
    .fetch_one(&state.pool)
    .await?;

    let rating_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM strain_ratings",
    )
    .fetch_one(&state.pool)
    .await?;

    let pending_vetting: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM strain_revisions WHERE status = 'pending'",
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "user_count": user_count,
        "strain_count": strain_count,
        "comment_count": comment_count,
        "rating_count": rating_count,
        "pending_vetting": pending_vetting,
    })))
}
