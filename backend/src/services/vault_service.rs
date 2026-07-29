// Vault service -- business logic for private strain management.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::{PrivateStrain, PrivateStrainRequest, StrainSummary};
use crate::services::strain_service;

/// List all strains in a user's vault (private strains + saved public strains).
pub async fn list_vault(
    pool: &PgPool,
    user_id: Uuid,
) -> AppResult<serde_json::Value> {
    // Fetch private strains
    let private = sqlx::query_as::<_, PrivateStrain>(
        "SELECT id, user_id, public_strain_id, name, \"type\"::text AS strain_type, \
                thc_percentage::float8, cbd_percentage::float8, \
                description, color, smell, flavor, breeder, lineage, \
                growing_difficulty::text, flowering_time_days, \
                personal_rating, personal_notes, created_at, updated_at \
         FROM private_strains WHERE user_id = $1 ORDER BY updated_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    // Fetch saved public strains
    let saved = sqlx::query_as::<_, StrainSummaryRow>(
        r#"SELECT ps.id, ps.name, ps.type::text AS strain_type,
                  ps.thc_percentage, ps.cbd_percentage,
                  ps.average_rating, ps.rating_count,
                  ps.created_at
           FROM public_strains ps
           JOIN user_saved_strains uss ON uss.strain_id = ps.id
           WHERE uss.user_id = $1 AND ps.is_active = true
           ORDER BY uss.saved_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let saved_strains: Vec<StrainSummary> = saved
        .into_iter()
        .map(|r| StrainSummary {
            id: r.id,
            name: r.name,
            strain_type: r.strain_type,
            thc_percentage: r.thc_percentage,
            cbd_percentage: r.cbd_percentage,
            average_rating: r.average_rating,
            rating_count: r.rating_count,
            thumbnail_url: None,
            created_at: None,
        })
        .collect();

    Ok(serde_json::json!({
        "private_strains": private,
        "saved_strains": saved_strains,
    }))
}

#[derive(Debug, sqlx::FromRow)]
struct StrainSummaryRow {
    id: Uuid,
    name: String,
    strain_type: String,
    thc_percentage: Option<f64>,
    cbd_percentage: Option<f64>,
    average_rating: Option<f64>,
    rating_count: i32,
    #[allow(dead_code)]
    created_at: chrono::DateTime<Utc>,
}

/// Create a new private strain in the user's vault.
pub async fn create_private_strain(
    pool: &PgPool,
    user_id: Uuid,
    req: &PrivateStrainRequest,
) -> AppResult<Uuid> {
    let strain_id = Uuid::new_v4();

    sqlx::query(
        r#"INSERT INTO private_strains
           (id, user_id, name, type, thc_percentage, cbd_percentage, description,
            color, smell, flavor, breeder, lineage, growing_difficulty,
            flowering_time_days, personal_rating, personal_notes)
           VALUES ($1, $2, $3, $4::strain_type, $5, $6, $7, $8, $9, $10, $11, $12, $13::growing_difficulty, $14, $15, $16)"#,
    )
    .bind(strain_id)
    .bind(user_id)
    .bind(&req.name)
    .bind(&req.strain_type)
    .bind(req.thc_percentage)
    .bind(req.cbd_percentage)
    .bind(&req.description)
    .bind(&req.color)
    .bind(&req.smell)
    .bind(&req.flavor)
    .bind(&req.breeder)
    .bind(&req.lineage)
    .bind(&req.growing_difficulty)
    .bind(req.flowering_time_days)
    .bind(req.personal_rating)
    .bind(&req.personal_notes)
    .execute(pool)
    .await?;

    Ok(strain_id)
}

/// Get a private strain detail (must be owned by the user).
pub async fn get_private_strain(
    pool: &PgPool,
    user_id: Uuid,
    strain_id: Uuid,
) -> AppResult<PrivateStrain> {
    let strain = sqlx::query_as::<_, PrivateStrain>(
        "SELECT id, user_id, public_strain_id, name, \"type\"::text AS strain_type, \
                thc_percentage::float8, cbd_percentage::float8, \
                description, color, smell, flavor, breeder, lineage, \
                growing_difficulty::text, flowering_time_days, \
                personal_rating, personal_notes, created_at, updated_at \
         FROM private_strains WHERE id = $1 AND user_id = $2",
    )
    .bind(strain_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Private strain not found".into()))?;

    Ok(strain)
}

/// Update a private strain (must be owned by the user).
pub async fn update_private_strain(
    pool: &PgPool,
    user_id: Uuid,
    strain_id: Uuid,
    req: &PrivateStrainRequest,
) -> AppResult<()> {
    let rows = sqlx::query(
        r#"UPDATE private_strains SET
            name = $1, type = $2::strain_type,
            thc_percentage = $3, cbd_percentage = $4,
            description = $5, color = $6, smell = $7, flavor = $8,
            breeder = $9, lineage = $10,
            growing_difficulty = $11::growing_difficulty,
            flowering_time_days = $12,
            personal_rating = $13, personal_notes = $14
           WHERE id = $15 AND user_id = $16"#,
    )
    .bind(&req.name)
    .bind(&req.strain_type)
    .bind(req.thc_percentage)
    .bind(req.cbd_percentage)
    .bind(&req.description)
    .bind(&req.color)
    .bind(&req.smell)
    .bind(&req.flavor)
    .bind(&req.breeder)
    .bind(&req.lineage)
    .bind(&req.growing_difficulty)
    .bind(req.flowering_time_days)
    .bind(req.personal_rating)
    .bind(&req.personal_notes)
    .bind(strain_id)
    .bind(user_id)
    .execute(pool)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound("Private strain not found".into()));
    }

    Ok(())
}

/// Delete a private strain (must be owned by the user).
pub async fn delete_private_strain(
    pool: &PgPool,
    user_id: Uuid,
    strain_id: Uuid,
) -> AppResult<()> {
    let rows = sqlx::query("DELETE FROM private_strains WHERE id = $1 AND user_id = $2")
        .bind(strain_id)
        .bind(user_id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound("Private strain not found".into()));
    }

    Ok(())
}

/// Push a private strain to the public catalog.
/// Creates an anonymous public copy (no user_id on public_strains).
/// Links the private strain to the new public strain via public_strain_id.
pub async fn push_to_public(
    pool: &PgPool,
    user_id: Uuid,
    private_strain_id: Uuid,
) -> AppResult<Uuid> {
    // Fetch the private strain
    let private = get_private_strain(pool, user_id, private_strain_id).await?;

    // Check if already pushed
    if private.public_strain_id.is_some() {
        return Err(AppError::Conflict(
            "This strain has already been pushed to the public catalog. Use push-update to sync changes.".into(),
        ));
    }

    // Check for name conflict in public catalog
    let name_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM public_strains WHERE LOWER(name) = LOWER($1)",
    )
    .bind(&private.name)
    .fetch_one(pool)
    .await?;

    if name_exists > 0 {
        return Err(AppError::Conflict(format!(
            "A strain named '{}' already exists in the public catalog. \
             If this is the same strain, please contribute your review and photos to the existing entry. \
             Otherwise, choose a different name for your strain.",
            private.name
        )));
    }

    // Create the public strain (no terpenes or effects from private -- those are public-only)
    let public_id = strain_service::create_strain(
        pool,
        &private.name,
        &private.strain_type,
        private.thc_percentage,
        private.cbd_percentage,
        private.description.as_deref(),
        private.color.as_deref(),
        private.smell.as_deref(),
        private.flavor.as_deref(),
        private.breeder.as_deref(),
        private.lineage.as_deref(),
        private.growing_difficulty.as_deref(),
        private.flowering_time_days,
        &[], // No terpenes from private
        &[], // No effects from private
    )
    .await?;

    // Link private to public
    sqlx::query("UPDATE private_strains SET public_strain_id = $1 WHERE id = $2")
        .bind(public_id)
        .bind(private_strain_id)
        .execute(pool)
        .await?;

    Ok(public_id)
}

/// Push updated fields from a private strain to its linked public strain.
/// Goes through the post-edit vetting process.
pub async fn push_update_to_public(
    pool: &PgPool,
    user_id: Uuid,
    private_strain_id: Uuid,
    change_summary: Option<&str>,
) -> AppResult<()> {
    let private = get_private_strain(pool, user_id, private_strain_id).await?;

    let public_id = private
        .public_strain_id
        .ok_or_else(|| AppError::BadRequest("This strain has not been pushed to the public catalog yet".into()))?;

    // Update the public strain (creates revision for vetting)
    strain_service::update_strain(
        pool,
        public_id,
        user_id,
        &private.name,
        &private.strain_type,
        private.thc_percentage,
        private.cbd_percentage,
        private.description.as_deref(),
        private.color.as_deref(),
        private.smell.as_deref(),
        private.flavor.as_deref(),
        private.breeder.as_deref(),
        private.lineage.as_deref(),
        private.growing_difficulty.as_deref(),
        private.flowering_time_days,
        &[], // Terpenes not managed in private
        &[], // Effects not managed in private
        change_summary,
    )
    .await?;

    Ok(())
}

/// Save a public strain to the user's vault (bookmark).
pub async fn save_public_strain(
    pool: &PgPool,
    user_id: Uuid,
    strain_id: Uuid,
) -> AppResult<()> {
    // Verify the strain exists
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM public_strains WHERE id = $1 AND is_active = true)",
    )
    .bind(strain_id)
    .fetch_one(pool)
    .await?;

    if !exists {
        return Err(AppError::NotFound("Strain not found".into()));
    }

    // Insert (ignore if already saved)
    sqlx::query(
        "INSERT INTO user_saved_strains (user_id, strain_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(strain_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Remove a saved public strain from the user's vault.
pub async fn unsave_public_strain(
    pool: &PgPool,
    user_id: Uuid,
    strain_id: Uuid,
) -> AppResult<()> {
    sqlx::query("DELETE FROM user_saved_strains WHERE user_id = $1 AND strain_id = $2")
        .bind(user_id)
        .bind(strain_id)
        .execute(pool)
        .await?;

    Ok(())
}
