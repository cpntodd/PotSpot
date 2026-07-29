use axum::{
    extract::{Multipart, Path, Query, State},
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
        .route("/:id", get(detail))
        .route("/:id", put(update))
        .route("/:id", delete(remove))
        .route("/:id/rate", post(rate))
        .route("/:id/similar", get(similar))
        .route("/:id/comments", get(get_comments))
        .route("/:id/comments", post(post_comment))
        .route("/:id/revisions", get(get_revisions))
        .route("/:id/photos", post(upload_photo))
        .route("/:id/photos", get(get_photos))
        .route("/:id/photos/:photo_id/rate", post(rate_photo))
        .route("/terpenes", get(list_terpenes))
        .route("/effects", get(list_effects))
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
/// Create a new public strain. Any authenticated user can create.
/// Submissions enter the vetting queue before appearing in the catalog.
async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
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
    auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_admin()?;
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

/// POST /api/v1/strains/:id/photos
///
/// Upload a photo for an existing strain. Multipart form with field `file`.
async fn upload_photo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(strain_id): Path<uuid::Uuid>,
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

        crate::services::photo_service::validate_photo(&data, &content_type, false)?;

        let photo_id = uuid::Uuid::new_v4();
        let s3_key = format!("strains/{}/{}.webp", strain_id, photo_id);
        let thumb_key = format!("strains/{}/{}_thumb.webp", strain_id, photo_id);

        // Strip EXIF, resize, upload
        let processed = crate::services::photo_service::strip_exif(&data)?;
        let thumbnail = crate::services::photo_service::generate_thumbnail(&processed)?;

        crate::s3::upload_object(&state.config, &s3_key, &processed, "image/webp").await?;
        crate::s3::upload_object(&state.config, &thumb_key, &thumbnail, "image/webp").await?;

        // Record in database
        sqlx::query(
            "INSERT INTO strain_photos (id, strain_id, user_id, s3_key, thumbnail_s3_key, content_type, file_size_bytes, width, height) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, 0, 0)",
        )
        .bind(photo_id)
        .bind(strain_id)
        .bind(auth.user_id)
        .bind(&s3_key)
        .bind(&thumb_key)
        .bind(&content_type)
        .bind(data.len() as i32)
        .execute(&state.pool)
        .await?;

        return Ok(Json(serde_json::json!({
            "message": "Photo uploaded",
            "photo_id": photo_id,
        })));
    }

    Err(AppError::BadRequest("No file provided".into()))
}

/// GET /api/v1/strains/:id/photos
///
/// Returns all photos for a strain with presigned URLs and rating info.
async fn get_photos(
    State(state): State<AppState>,
    Path(strain_id): Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let photos = crate::services::strain_service::get_strain_photos(&state.pool, strain_id).await?;

    let mut result = Vec::new();
    for p in photos {
        let photo_url = crate::s3::presign_get_url(&state.config, &p.s3_key).await.ok();
        let thumb_url = if let Some(ref tk) = p.thumbnail_s3_key {
            crate::s3::presign_get_url(&state.config, tk).await.ok()
        } else {
            None
        };
        result.push(serde_json::json!({
            "id": p.id,
            "s3_key": p.s3_key,
            "photo_url": photo_url,
            "thumbnail_url": thumb_url,
            "content_type": p.content_type,
            "width": p.width,
            "height": p.height,
            "is_primary": p.is_primary,
            "average_rating": p.average_rating,
            "rating_count": p.rating_count,
            "user_id": p.user_id,
            "created_at": p.created_at,
        }));
    }

    Ok(Json(serde_json::json!(result)))
}

/// POST /api/v1/strains/:id/photos/:photo_id/rate
///
/// Rate a photo from 1 to 5. One rating per user per photo.
async fn rate_photo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((strain_id, photo_id)): Path<(uuid::Uuid, uuid::Uuid)>,
    Json(req): Json<RateStrainRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if req.rating < 1 || req.rating > 5 {
        return Err(AppError::BadRequest("Rating must be between 1 and 5".into()));
    }

    // Verify the photo belongs to this strain
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM strain_photos WHERE id = $1 AND strain_id = $2",
    )
    .bind(photo_id)
    .bind(strain_id)
    .fetch_one(&state.pool)
    .await?;

    if count.0 == 0 {
        return Err(AppError::NotFound("Photo not found for this strain".into()));
    }

    // Upsert rating
    sqlx::query(
        r#"INSERT INTO photo_ratings (photo_id, user_id, rating)
           VALUES ($1, $2, $3)
           ON CONFLICT (photo_id, user_id)
           DO UPDATE SET rating = $3, created_at = NOW()"#,
    )
    .bind(photo_id)
    .bind(auth.user_id)
    .bind(req.rating)
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "message": "Photo rating submitted successfully",
    })))
}

/// GET /api/v1/strains/terpenes
async fn list_terpenes(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<crate::models::TerpeneInfo>>> {
    let terpenes = crate::services::strain_service::list_terpenes(&state.pool).await?;
    Ok(Json(terpenes))
}

/// GET /api/v1/strains/effects
async fn list_effects(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<crate::models::EffectInfo>>> {
    let effects = crate::services::strain_service::list_effects(&state.pool).await?;
    Ok(Json(effects))
}
