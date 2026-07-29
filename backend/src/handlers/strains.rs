use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};

use crate::auth::middleware::AuthUser;
use crate::errors::{AppError, AppResult};
use crate::models::{
    CreateStrainRequest, RateStrainRequest, StrainListResponse, StrainSearchQuery, UpdateStrainRequest,
};
use crate::services::{comment_service, similarity, strain_service, vetting_service};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/", post(create))
        .route("/{id}", get(detail))
        .route("/{id}", put(update))
        .route("/{id}", delete(remove))
        .route("/{id}/rate", post(rate))
        .route("/{id}/similar", get(similar))
        .route("/{id}/comments", get(get_comments))
        .route("/{id}/comments", post(post_comment))
        .route("/{id}/revisions", get(get_revisions))
}

/// GET /api/v1/strains
///
/// List strains with full-text search, filters, sorting, and pagination.
async fn list(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<StrainSearchQuery>,
) -> AppResult<Json<StrainListResponse>> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    let (strains, total) = strain_service::search_strains(&state.pool, &query).await?;

    Ok(Json(StrainListResponse {
        strains,
        total,
        page,
        per_page,
    }))
}

/// GET /api/v1/strains/:id
///
/// Get full strain detail with terpenes, effects, and photo URL.
async fn detail(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let mut detail = strain_service::get_strain_detail(&state.pool, id).await?;

    // Generate presigned URL for the primary photo if one exists
    let photo_url = if let Some(ref key) = detail.primary_photo_key {
        crate::s3::presign_get_url(&state.config, key).await.ok()
    } else {
        None
    };

    // Build the response JSON manually since primary_photo_key is skipped in serialization
    let response = serde_json::json!({
        "id": detail.strain.id,
        "name": detail.strain.name,
        "type": detail.strain.strain_type,
        "thc_percentage": detail.strain.thc_percentage,
        "cbd_percentage": detail.strain.cbd_percentage,
        "description": detail.strain.description,
        "color": detail.strain.color,
        "smell": detail.strain.smell,
        "flavor": detail.strain.flavor,
        "breeder": detail.strain.breeder,
        "lineage": detail.strain.lineage,
        "growing_difficulty": detail.strain.growing_difficulty,
        "flowering_time_days": detail.strain.flowering_time_days,
        "average_rating": detail.strain.average_rating,
        "rating_count": detail.strain.rating_count,
        "created_at": detail.strain.created_at,
        "updated_at": detail.strain.updated_at,
        "version": detail.strain.version,
        "terpenes": detail.terpenes,
        "effects": detail.effects,
        "primary_photo_url": photo_url,
    });

    Ok(Json(response))
}

/// POST /api/v1/strains
///
/// Create a new public strain. Admin only for now.
async fn create(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<CreateStrainRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: Check admin role when middleware is wired
    // For now, any authenticated user can create (will be restricted later)

    let strain_id = strain_service::create_strain(
        &state.pool,
        &req.name,
        &req.strain_type,
        req.thc_percentage,
        req.cbd_percentage,
        req.description.as_deref(),
        req.color.as_deref(),
        req.smell.as_deref(),
        req.flavor.as_deref(),
        req.breeder.as_deref(),
        req.lineage.as_deref(),
        req.growing_difficulty.as_deref(),
        req.flowering_time_days,
        &req.terpene_ids,
        &req.effect_ids,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "message": "Strain created successfully",
        "id": strain_id,
    })))
}

/// PUT /api/v1/strains/:id
///
/// Update a public strain. Creates a pending revision for vetting.
/// The edit goes live immediately (post-edit vetting model).
async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<UpdateStrainRequest>,
) -> AppResult<Json<serde_json::Value>> {
    strain_service::update_strain(
        &state.pool,
        id,
        auth.user_id,
        &req.name,
        &req.strain_type,
        req.thc_percentage,
        req.cbd_percentage,
        req.description.as_deref(),
        req.color.as_deref(),
        req.smell.as_deref(),
        req.flavor.as_deref(),
        req.breeder.as_deref(),
        req.lineage.as_deref(),
        req.growing_difficulty.as_deref(),
        req.flowering_time_days,
        &req.terpene_ids,
        &req.effect_ids,
        req.change_summary.as_deref(),
    )
    .await?;

    Ok(Json(serde_json::json!({
        "message": "Strain updated. Changes are pending vetting review.",
    })))
}

/// DELETE /api/v1/strains/:id
///
/// Deactivate a public strain. Admin only.
async fn remove(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    strain_service::deactivate_strain(&state.pool, id).await?;

    Ok(Json(serde_json::json!({
        "message": "Strain deactivated successfully",
    })))
}

/// POST /api/v1/strains/:id/rate
///
/// Rate a strain from 1 to 5. One rating per user per strain.
/// Invalidates the similarity cache for this strain.
async fn rate(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(strain_id): Path<uuid::Uuid>,
    Json(req): Json<RateStrainRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if req.rating < 1 || req.rating > 5 {
        return Err(AppError::BadRequest("Rating must be between 1 and 5".into()));
    }

    // Upsert: insert or update the user's rating
    sqlx::query(
        r#"INSERT INTO strain_ratings (strain_id, user_id, rating)
           VALUES ($1, $2, $3)
           ON CONFLICT (strain_id, user_id)
           DO UPDATE SET rating = $3, created_at = NOW()"#,
    )
    .bind(strain_id)
    .bind(auth.user_id)
    .bind(req.rating)
    .execute(&state.pool)
    .await?;

    // Invalidate similarity cache (ratings affect collaborative filtering)
    similarity::invalidate_cache(strain_id);

    Ok(Json(serde_json::json!({
        "message": "Rating submitted successfully",
    })))
}

/// GET /api/v1/strains/:id/comments
async fn get_comments(
    State(state): State<AppState>,
    Path(strain_id): Path<uuid::Uuid>,
    auth: Option<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let current_user_id = auth.map(|u| u.user_id);
    let comments = comment_service::get_comments(&state.pool, strain_id, current_user_id).await?;
    Ok(Json(serde_json::to_value(comments).map_err(|e| AppError::Internal(e.into()))?))
}

/// POST /api/v1/strains/:id/comments
#[derive(serde::Deserialize)]
struct PostCommentBody {
    body: String,
}

async fn post_comment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(strain_id): Path<uuid::Uuid>,
    Json(req): Json<PostCommentBody>,
) -> AppResult<Json<serde_json::Value>> {
    let id = comment_service::post_comment(
        &state.pool,
        strain_id,
        auth.user_id,
        None, // top-level comment
        &req.body,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "message": "Comment posted",
        "id": id,
    })))
}

/// GET /api/v1/strains/:id/revisions
async fn get_revisions(
    State(state): State<AppState>,
    Path(strain_id): Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let revisions = vetting_service::get_version_history(&state.pool, strain_id).await?;
    Ok(Json(serde_json::to_value(revisions).map_err(|e| AppError::Internal(e.into()))?))
}

/// GET /api/v1/strains/:id/similar
///
/// Returns strains similar to the given strain based on terpene profile,
/// effect profile, type, and collaborative filtering.
async fn similar(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<Json<Vec<crate::models::StrainSummary>>> {
    let strains = similarity::get_similar_strains(&state.pool, id, 10).await?;
    Ok(Json(strains))
}

