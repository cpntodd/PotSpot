// Vetting service -- collaborative quality control for public strain edits.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

/// A revision entry for the vetting queue.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RevisionEntry {
    pub id: Uuid,
    pub strain_id: Uuid,
    pub strain_name: String,
    pub proposed_by: Uuid,
    pub proposed_by_name: String,
    pub change_summary: Option<String>,
    pub old_data: serde_json::Value,
    pub new_data: serde_json::Value,
    pub status: String,
    pub vetted_by: Option<Uuid>,
    pub vetted_by_name: Option<String>,
    pub vetted_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Get the pending vetting queue (only for vetter+ roles).
pub async fn get_pending_queue(pool: &PgPool) -> AppResult<Vec<RevisionEntry>> {
    let revisions = sqlx::query_as::<_, RevisionEntry>(
        r#"SELECT
            sr.id, sr.strain_id, ps.name AS strain_name,
            sr.proposed_by, u.display_name AS proposed_by_name,
            sr.change_summary, sr.old_data, sr.new_data,
            sr.status::text AS status,
            sr.vetted_by, vu.display_name AS vetted_by_name,
            sr.vetted_at, sr.rejection_reason, sr.created_at
           FROM strain_revisions sr
           JOIN public_strains ps ON ps.id = sr.strain_id
           JOIN users u ON u.id = sr.proposed_by
           LEFT JOIN users vu ON vu.id = sr.vetted_by
           WHERE sr.status = 'pending'
           ORDER BY sr.created_at ASC"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(revisions)
}

/// Approve a pending revision. Marks it as approved.
/// The edit is already live (post-edit model), so this just confirms it.
pub async fn approve_revision(
    pool: &PgPool,
    revision_id: Uuid,
    vetted_by: Uuid,
) -> AppResult<()> {
    let rows = sqlx::query(
        r#"UPDATE strain_revisions
           SET status = 'approved', vetted_by = $1, vetted_at = NOW()
           WHERE id = $2 AND status = 'pending'"#,
    )
    .bind(vetted_by)
    .bind(revision_id)
    .execute(pool)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(
            "Revision not found or already processed".into(),
        ));
    }

    Ok(())
}

/// Reject a pending revision. Rolls back the strain to old_data.
pub async fn reject_revision(
    pool: &PgPool,
    revision_id: Uuid,
    vetted_by: Uuid,
    reason: &str,
) -> AppResult<()> {
    // Fetch the revision
    let revision = sqlx::query_as::<_, RevisionEntry>(
        r#"SELECT
            sr.id, sr.strain_id, ps.name AS strain_name,
            sr.proposed_by, u.display_name AS proposed_by_name,
            sr.change_summary, sr.old_data, sr.new_data,
            sr.status::text AS status,
            sr.vetted_by, vu.display_name AS vetted_by_name,
            sr.vetted_at, sr.rejection_reason, sr.created_at
           FROM strain_revisions sr
           JOIN public_strains ps ON ps.id = sr.strain_id
           JOIN users u ON u.id = sr.proposed_by
           LEFT JOIN users vu ON vu.id = sr.vetted_by
           WHERE sr.id = $1 AND sr.status = 'pending'"#,
    )
    .bind(revision_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Revision not found or already processed".into()))?;

    // Rollback: restore the strain from old_data
    rollback_strain(pool, &revision).await?;

    // Mark revision as rejected
    sqlx::query(
        r#"UPDATE strain_revisions
           SET status = 'rejected', vetted_by = $1, vetted_at = NOW(),
               rejection_reason = $2
           WHERE id = $3"#,
    )
    .bind(vetted_by)
    .bind(reason)
    .bind(revision_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Rollback a strain to its previous state from the revision's old_data snapshot.
async fn rollback_strain(pool: &PgPool, revision: &RevisionEntry) -> AppResult<()> {
    let old = &revision.old_data;

    // Extract fields from old_data JSONB
    let name = old["name"].as_str().unwrap_or("Unknown");
    let strain_type = old["strain_type"].as_str().unwrap_or("hybrid");
    let thc = old["thc_percentage"].as_f64();
    let cbd = old["cbd_percentage"].as_f64();
    let description = old["description"].as_str();
    let color = old["color"].as_str();
    let smell = old["smell"].as_str();
    let flavor = old["flavor"].as_str();
    let breeder = old["breeder"].as_str();
    let lineage = old["lineage"].as_str();
    let growing_difficulty = old["growing_difficulty"].as_str();
    let flowering_time_days = old["flowering_time_days"].as_i64().map(|v| v as i16);

    sqlx::query(
        r#"UPDATE public_strains SET
            name = $1, type = $2::strain_type,
            thc_percentage = $3, cbd_percentage = $4,
            description = $5, color = $6, smell = $7, flavor = $8,
            breeder = $9, lineage = $10,
            growing_difficulty = $11::growing_difficulty,
            flowering_time_days = $12,
            version = version + 1
           WHERE id = $13"#,
    )
    .bind(name)
    .bind(strain_type)
    .bind(thc)
    .bind(cbd)
    .bind(description)
    .bind(color)
    .bind(smell)
    .bind(flavor)
    .bind(breeder)
    .bind(lineage)
    .bind(growing_difficulty)
    .bind(flowering_time_days)
    .bind(revision.strain_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get the full version history for a strain (all revisions, any status).
pub async fn get_version_history(
    pool: &PgPool,
    strain_id: Uuid,
) -> AppResult<Vec<RevisionEntry>> {
    // Verify strain exists
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM public_strains WHERE id = $1)",
    )
    .bind(strain_id)
    .fetch_one(pool)
    .await?;

    if !exists {
        return Err(AppError::NotFound("Strain not found".into()));
    }

    let revisions = sqlx::query_as::<_, RevisionEntry>(
        r#"SELECT
            sr.id, sr.strain_id, ps.name AS strain_name,
            sr.proposed_by, u.display_name AS proposed_by_name,
            sr.change_summary, sr.old_data, sr.new_data,
            sr.status::text AS status,
            sr.vetted_by, vu.display_name AS vetted_by_name,
            sr.vetted_at, sr.rejection_reason, sr.created_at
           FROM strain_revisions sr
           JOIN public_strains ps ON ps.id = sr.strain_id
           JOIN users u ON u.id = sr.proposed_by
           LEFT JOIN users vu ON vu.id = sr.vetted_by
           WHERE sr.strain_id = $1
           ORDER BY sr.created_at DESC"#,
    )
    .bind(strain_id)
    .fetch_all(pool)
    .await?;

    Ok(revisions)
}
