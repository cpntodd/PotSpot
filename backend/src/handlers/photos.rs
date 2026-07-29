use axum::{
    extract::{Multipart, Path, State},
    response::Redirect,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::errors::{AppError, AppResult};
use crate::services::photo_service;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/upload", post(upload))
        .route("/{id}", get(serve))
}

/// POST /api/v1/photos/upload
///
/// Upload a strain photo. Multipart form with fields:
/// - `strain_id` (text): UUID of the strain
/// - `file` (binary): The image file
/// - `is_primary` (text, optional): "true" to set as primary photo
async fn upload(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    let mut strain_id: Option<Uuid> = None;
    let mut is_primary = false;
    let mut file_data: Option<Vec<u8>> = None;
    let mut content_type: Option<String> = None;

    // Parse multipart fields
    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "strain_id" => {
                let text = field.text().await.map_err(|e| {
                    AppError::BadRequest(format!("Failed to read strain_id: {}", e))
                })?;
                strain_id = Some(Uuid::parse_str(&text).map_err(|_| {
                    AppError::BadRequest("Invalid strain_id UUID".into())
                })?);
            }
            "is_primary" => {
                let text = field.text().await.map_err(|e| {
                    AppError::BadRequest(format!("Failed to read is_primary: {}", e))
                })?;
                is_primary = text == "true";
            }
            "file" => {
                content_type = field.content_type().map(|s| s.to_string());
                let bytes = field.bytes().await.map_err(|e| {
                    AppError::BadRequest(format!("Failed to read file: {}", e))
                })?;
                file_data = Some(bytes.to_vec());
            }
            _ => {}
        }
    }

    // Validate required fields
    let strain_id = strain_id.ok_or_else(|| AppError::BadRequest("Missing strain_id field".into()))?;
    let file_data = file_data.ok_or_else(|| AppError::BadRequest("Missing file field".into()))?;
    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".into());

    // Verify the strain exists
    let strain_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM public_strains WHERE id = $1 AND is_active = true)",
    )
    .bind(strain_id)
    .fetch_one(&state.pool)
    .await?;

    if !strain_exists {
        return Err(AppError::NotFound("Strain not found".into()));
    }

    // Validate the photo
    photo_service::validate_photo(&file_data, &content_type, is_primary)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Strip EXIF metadata
    let cleaned = photo_service::strip_exif(&file_data)
        .map_err(|e| AppError::Internal(e))?;

    // Generate thumbnail
    let thumbnail = photo_service::generate_thumbnail(&cleaned)
        .map_err(|e| AppError::Internal(e))?;

    // Get dimensions
    let (width, height) = photo_service::get_dimensions(&cleaned)
        .map_err(|e| AppError::Internal(e))?;

    // Generate object keys
    let photo_id = Uuid::new_v4();
    let s3_key = format!("strains/{}/photos/{}.webp", strain_id, photo_id);
    let thumb_key = format!("strains/{}/thumbs/{}.webp", strain_id, photo_id);

    // Upload to MinIO
    crate::s3::upload_object(&state.config, &s3_key, &cleaned, "image/webp")
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to upload photo to MinIO");
            AppError::Internal(e)
        })?;

    crate::s3::upload_object(&state.config, &thumb_key, &thumbnail, "image/webp")
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to upload thumbnail to MinIO");
            AppError::Internal(e)
        })?;

    // If this is set as primary, clear existing primary first
    if is_primary {
        sqlx::query("UPDATE strain_photos SET is_primary = false WHERE strain_id = $1")
            .bind(strain_id)
            .execute(&state.pool)
            .await?;
    }

    // Store metadata
    sqlx::query(
        r#"INSERT INTO strain_photos
           (id, strain_id, user_id, is_primary, s3_key, thumbnail_s3_key,
            content_type, file_size_bytes, width, height)
           VALUES ($1, $2, $3, $4, $5, $6, 'image/webp', $7, $8, $9)"#,
    )
    .bind(photo_id)
    .bind(strain_id)
    .bind(auth.user_id)
    .bind(is_primary)
    .bind(&s3_key)
    .bind(&thumb_key)
    .bind(cleaned.len() as i32)
    .bind(width as i16)
    .bind(height as i16)
    .execute(&state.pool)
    .await?;

    // Generate presigned URL for the uploaded photo
    let presigned_url = crate::s3::presign_get_url(&state.config, &s3_key).await.ok();

    Ok(Json(serde_json::json!({
        "message": "Photo uploaded successfully",
        "id": photo_id,
        "url": presigned_url,
        "width": width,
        "height": height,
    })))
}

/// GET /api/v1/photos/:id
///
/// Redirect to a presigned MinIO URL for the photo.
async fn serve(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Redirect> {
    let s3_key: Option<String> = sqlx::query_scalar(
        "SELECT s3_key FROM strain_photos WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;

    let key = s3_key.ok_or_else(|| AppError::NotFound("Photo not found".into()))?;

    let url = crate::s3::presign_get_url(&state.config, &key)
        .await
        .map_err(|e| AppError::Internal(e))?;

    Ok(Redirect::temporary(&url))
}

