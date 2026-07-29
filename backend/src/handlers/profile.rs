use axum::{
    extract::{Multipart, Path, State},
    routing::{get, post, put},
    Json, Router,
};

use crate::auth::middleware::AuthUser;
use crate::errors::{AppError, AppResult};
use crate::models::{ProfileStats, ProfileUpdateRequest, UserProfile};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(profile))
        .route("/", put(update_profile))
        .route("/stats", get(stats))
        .route("/strains", get(my_strains))
        .route("/comments", get(my_comments))
        .route("/reviews", get(my_reviews))
        .route("/saved", get(my_saved))
        .route("/avatar", post(upload_avatar))
        .route("/banner", post(upload_banner))
}

/// GET /api/v1/profile
async fn profile(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT id, email, password_hash, display_name, role::text, \
                age_verified, date_of_birth, avatar_s3_key, banner_s3_key, bio, \
                created_at, updated_at \
         FROM users WHERE id = $1",
    )
    .bind(auth.user_id)
    .fetch_one(&state.pool)
    .await?;

    // Generate presigned URLs for avatar and banner
    let avatar_url = if let Some(ref key) = user.avatar_s3_key {
        crate::s3::presign_get_url(&state.config, key).await.ok()
    } else {
        None
    };

    let banner_url = if let Some(ref key) = user.banner_s3_key {
        crate::s3::presign_get_url(&state.config, key).await.ok()
    } else {
        None
    };

    // Stats
    let stats = get_user_stats(&state.pool, auth.user_id).await?;

    Ok(Json(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "display_name": user.display_name,
        "role": user.role,
        "bio": user.bio,
        "avatar_url": avatar_url,
        "banner_url": banner_url,
        "age_verified": user.age_verified,
        "created_at": user.created_at,
        "stats": {
            "strains_submitted": stats.strains_submitted,
            "strains_in_vault": stats.strains_in_vault,
            "comments": stats.comments,
            "reviews": stats.reviews,
            "saved_strains": stats.saved_strains,
        },
    })))
}

/// GET /api/v1/profile/stats
async fn stats(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<ProfileStats>> {
    let stats = get_user_stats(&state.pool, auth.user_id).await?;
    Ok(Json(stats))
}

/// GET /api/v1/profile/strains?type=public|private
async fn my_strains(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<serde_json::Value>> {
    let filter = params.get("type").map(|s| s.as_str()).unwrap_or("all");

    let public_strains = if filter == "public" || filter == "all" {
        Some(
            sqlx::query_as::<_, crate::models::StrainSummary>(
                r#"SELECT ps.id, ps.name, ps.type::text AS strain_type,
                          ps.thc_percentage::float8, ps.cbd_percentage::float8,
                          ps.average_rating::float8, ps.rating_count,
                          ps.created_at
                   FROM public_strains ps
                   JOIN strain_revisions sr ON sr.strain_id = ps.id
                   WHERE sr.proposed_by = $1 AND sr.status = 'approved'
                   ORDER BY ps.created_at DESC
                   LIMIT 50"#,
            )
            .bind(auth.user_id)
            .fetch_all(&state.pool)
            .await?,
        )
    } else {
        None
    };

    let private_strains = if filter == "private" || filter == "all" {
        Some(
            sqlx::query_as::<_, crate::models::PrivateStrain>(
                "SELECT id, user_id, public_strain_id, name, \"type\"::text AS strain_type, \
                        thc_percentage::float8, cbd_percentage::float8, \
                        description, color, smell, flavor, breeder, lineage, \
                        growing_difficulty::text, flowering_time_days, \
                        personal_rating, personal_notes, created_at, updated_at \
                 FROM private_strains WHERE user_id = $1 ORDER BY updated_at DESC LIMIT 50",
            )
            .bind(auth.user_id)
            .fetch_all(&state.pool)
            .await?,
        )
    } else {
        None
    };

    Ok(Json(serde_json::json!({
        "public": public_strains,
        "private": private_strains,
    })))
}

/// PUT /api/v1/profile
async fn update_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ProfileUpdateRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if let Some(ref name) = req.display_name {
        if name.trim().is_empty() {
            return Err(AppError::BadRequest("Display name cannot be empty".into()));
        }
        sqlx::query("UPDATE users SET display_name = $1, updated_at = NOW() WHERE id = $2")
            .bind(name.trim())
            .bind(auth.user_id)
            .execute(&state.pool)
            .await?;
    }

    if let Some(ref bio) = req.bio {
        sqlx::query("UPDATE users SET bio = $1, updated_at = NOW() WHERE id = $2")
            .bind(bio)
            .bind(auth.user_id)
            .execute(&state.pool)
            .await?;
    }

    Ok(Json(serde_json::json!({
        "message": "Profile updated",
    })))
}

/// POST /api/v1/profile/avatar
async fn upload_avatar(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Invalid multipart data: {}", e))
    })? {
        let data = field.bytes().await.map_err(|e| {
            AppError::BadRequest(format!("Failed to read upload: {}", e))
        })?;

        crate::services::photo_service::validate_photo(&data, "image/webp", true)?;

        let s3_key = format!("avatars/{}.webp", auth.user_id);
        crate::s3::upload_object(
            &state.config,
            &s3_key,
            &data,
            "image/webp",
        )
        .await?;

        sqlx::query("UPDATE users SET avatar_s3_key = $1, updated_at = NOW() WHERE id = $2")
            .bind(&s3_key)
            .bind(auth.user_id)
            .execute(&state.pool)
            .await?;

        let url = crate::s3::presign_get_url(&state.config, &s3_key).await.ok();

        return Ok(Json(serde_json::json!({
            "message": "Avatar uploaded",
            "avatar_url": url,
        })));
    }

    Err(AppError::BadRequest("No file provided".into()))
}

/// POST /api/v1/profile/banner
async fn upload_banner(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Invalid multipart data: {}", e))
    })? {
        let content_type = field
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "image/jpeg".into());

        let data = field.bytes().await.map_err(|e| {
            AppError::BadRequest(format!("Failed to read upload: {}", e))
        })?;

        crate::services::photo_service::validate_photo(&data, &content_type, true)?;

        let s3_key = format!("banners/{}.webp", auth.user_id);
        crate::s3::upload_object(
            &state.config,
            &s3_key,
            &data,
            &content_type,
        )
        .await?;

        sqlx::query("UPDATE users SET banner_s3_key = $1, updated_at = NOW() WHERE id = $2")
            .bind(&s3_key)
            .bind(auth.user_id)
            .execute(&state.pool)
            .await?;

        let url = crate::s3::presign_get_url(&state.config, &s3_key).await.ok();

        return Ok(Json(serde_json::json!({
            "message": "Banner uploaded",
            "banner_url": url,
        })));
    }

    Err(AppError::BadRequest("No file provided".into()))
}

/// Helper: compute user stats.
async fn get_user_stats(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> AppResult<ProfileStats> {
    let strains_submitted: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM strain_revisions WHERE proposed_by = $1 AND status = 'approved'",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let strains_in_vault: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM private_strains WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let comments: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM comments WHERE user_id = $1 AND is_deleted = false",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let reviews: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM strain_ratings WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let saved_strains: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM saved_strains WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(ProfileStats {
        strains_submitted,
        strains_in_vault,
        comments,
        reviews,
        saved_strains,
    })
}

/// GET /api/v1/profile/comments
async fn my_comments(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    let comments = sqlx::query_as::<_, (uuid::Uuid, String, uuid::Uuid, chrono::DateTime<chrono::Utc>)>(
        "SELECT c.id, c.body, c.strain_id, c.created_at \
         FROM comments c \
         WHERE c.user_id = $1 AND c.is_deleted = false \
         ORDER BY c.created_at DESC LIMIT 50",
    )
    .bind(auth.user_id)
    .fetch_all(&state.pool)
    .await?;

    let list: Vec<serde_json::Value> = comments
        .into_iter()
        .map(|(id, body, strain_id, created_at)| {
            serde_json::json!({
                "id": id,
                "body": body,
                "strain_id": strain_id,
                "created_at": created_at,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "comments": list })))
}

/// GET /api/v1/profile/reviews
async fn my_reviews(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    let reviews = sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid, i16, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, strain_id, rating, created_at \
         FROM strain_ratings WHERE user_id = $1 \
         ORDER BY created_at DESC LIMIT 50",
    )
    .bind(auth.user_id)
    .fetch_all(&state.pool)
    .await?;

    let list: Vec<serde_json::Value> = reviews
        .into_iter()
        .map(|(id, strain_id, rating, created_at)| {
            serde_json::json!({
                "id": id,
                "strain_id": strain_id,
                "rating": rating,
                "created_at": created_at,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "reviews": list })))
}

/// GET /api/v1/profile/saved
async fn my_saved(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    let saved = sqlx::query_as::<_, crate::models::StrainSummary>(
        r#"SELECT ps.id, ps.name, ps.type::text AS strain_type,
                  ps.thc_percentage::float8, ps.cbd_percentage::float8,
                  ps.average_rating::float8, ps.rating_count,
                  ps.created_at
           FROM public_strains ps
           JOIN saved_strains ss ON ss.strain_id = ps.id
           WHERE ss.user_id = $1 AND ps.is_active = true
           ORDER BY ss.created_at DESC LIMIT 50"#,
    )
    .bind(auth.user_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "saved": saved })))
}
