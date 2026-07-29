// Strain service -- business logic for strain operations.

use chrono::Utc;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::{
    EffectInfo, PublicStrain, StrainDetail, StrainSearchQuery, StrainSummary, TerpeneInfo,
};

/// Search strains with full-text search, filters, and pagination.
pub async fn search_strains(
    pool: &PgPool,
    query: &StrainSearchQuery,
) -> AppResult<(Vec<StrainSummary>, i64)> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    // --- Build COUNT query ---
    let mut count_builder = QueryBuilder::<Postgres>::new(
        "SELECT COUNT(*) FROM public_strains ps WHERE ps.is_active = true",
    );

    // --- Build data query ---
    let mut data_builder = QueryBuilder::<Postgres>::new(
        r#"SELECT
            ps.id, ps.name, ps.type::text AS strain_type,
            ps.thc_percentage::float8, ps.cbd_percentage::float8,
            ps.average_rating::float8, ps.rating_count,
            ps.created_at
        FROM public_strains ps
        WHERE ps.is_active = true"#,
    );

    // Full-text search
    if let Some(ref q) = query.q {
        if !q.trim().is_empty() {
            let fts_clause =
                " AND to_tsvector('english', ps.name || ' ' || COALESCE(ps.description, '')) \
                 @@ websearch_to_tsquery('english', ";
            count_builder.push(fts_clause);
            data_builder.push(fts_clause);
            count_builder.push_bind(q.trim());
            data_builder.push_bind(q.trim());
            count_builder.push(")");
            data_builder.push(")");
        }
    }

    // Type filter
    if let Some(ref strain_type) = query.strain_type {
        count_builder.push(" AND ps.type = ").push_bind(strain_type);
        data_builder.push(" AND ps.type = ").push_bind(strain_type);
    }

    // THC range
    if let Some(thc_min) = query.thc_min {
        count_builder.push(" AND ps.thc_percentage >= ").push_bind(thc_min);
        data_builder.push(" AND ps.thc_percentage >= ").push_bind(thc_min);
    }
    if let Some(thc_max) = query.thc_max {
        count_builder.push(" AND ps.thc_percentage <= ").push_bind(thc_max);
        data_builder.push(" AND ps.thc_percentage <= ").push_bind(thc_max);
    }

    // CBD range
    if let Some(cbd_min) = query.cbd_min {
        count_builder.push(" AND ps.cbd_percentage >= ").push_bind(cbd_min);
        data_builder.push(" AND ps.cbd_percentage >= ").push_bind(cbd_min);
    }
    if let Some(cbd_max) = query.cbd_max {
        count_builder.push(" AND ps.cbd_percentage <= ").push_bind(cbd_max);
        data_builder.push(" AND ps.cbd_percentage <= ").push_bind(cbd_max);
    }

    // Rating minimum
    if let Some(rating_min) = query.rating_min {
        count_builder.push(" AND ps.average_rating >= ").push_bind(rating_min);
        data_builder.push(" AND ps.average_rating >= ").push_bind(rating_min);
    }

    // Terpene filter (strain must have ALL specified terpenes)
    if let Some(ref terpenes_str) = query.terpenes {
        let ids: Vec<i16> = parse_comma_separated_i16(terpenes_str)?;
        if !ids.is_empty() {
            let subquery = format!(
                " AND ps.id IN (SELECT strain_id FROM strain_terpenes \
                 WHERE terpene_id = ANY(ARRAY[{}]::smallint[]) \
                 GROUP BY strain_id HAVING COUNT(DISTINCT terpene_id) = {})",
                ids.iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                ids.len()
            );
            count_builder.push(&subquery);
            data_builder.push(&subquery);
        }
    }

    // Effect filter (strain must have ALL specified effects)
    if let Some(ref effects_str) = query.effects {
        let ids: Vec<i16> = parse_comma_separated_i16(effects_str)?;
        if !ids.is_empty() {
            let subquery = format!(
                " AND ps.id IN (SELECT strain_id FROM strain_effects \
                 WHERE effect_id = ANY(ARRAY[{}]::smallint[]) \
                 GROUP BY strain_id HAVING COUNT(DISTINCT effect_id) = {})",
                ids.iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                ids.len()
            );
            count_builder.push(&subquery);
            data_builder.push(&subquery);
        }
    }

    // --- Sorting ---
    let sort = query.sort.as_deref().unwrap_or("newest");
    let order = query.order.as_deref().unwrap_or("desc");
    let ascending = order == "asc";

    match sort {
        "rating" => {
            data_builder.push(if ascending {
                " ORDER BY ps.average_rating ASC NULLS FIRST"
            } else {
                " ORDER BY ps.average_rating DESC NULLS LAST"
            });
        }
        "name" => {
            data_builder.push(if ascending {
                " ORDER BY ps.name ASC"
            } else {
                " ORDER BY ps.name DESC"
            });
        }
        "thc" => {
            data_builder.push(if ascending {
                " ORDER BY ps.thc_percentage ASC NULLS FIRST"
            } else {
                " ORDER BY ps.thc_percentage DESC NULLS LAST"
            });
        }
        _ => {
            data_builder.push(if ascending {
                " ORDER BY ps.created_at ASC"
            } else {
                " ORDER BY ps.created_at DESC"
            });
        }
    }

    // --- Pagination ---
    let offset = (page - 1) * per_page;
    data_builder.push(" LIMIT ").push_bind(per_page);
    data_builder.push(" OFFSET ").push_bind(offset);

    // Execute count
    let total: i64 = count_builder.build_query_scalar().fetch_one(pool).await?;

    // Execute data query
    let rows: Vec<StrainSummaryRow> = data_builder.build_query_as().fetch_all(pool).await?;

    let strains: Vec<StrainSummary> = rows
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

    Ok((strains, total))
}

/// Raw row type for the search query result.
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

/// Get full strain detail with terpenes, effects, and primary photo key.
pub async fn get_strain_detail(pool: &PgPool, strain_id: Uuid) -> AppResult<StrainDetail> {
    let strain = sqlx::query_as::<_, PublicStrain>(
        r#"SELECT id, name, type::text, thc_percentage::float8, cbd_percentage::float8,
                  description, color, smell, flavor, breeder, lineage,
                  growing_difficulty::text, flowering_time_days,
                  average_rating::float8, rating_count, is_active,
                  created_at, updated_at, version
           FROM public_strains WHERE id = $1 AND is_active = true"#,
    )
    .bind(strain_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Strain not found".into()))?;

    let terpenes = sqlx::query_as::<_, TerpeneInfo>(
        r#"SELECT t.id, t.name, t.icon, t.description
           FROM terpenes t
           JOIN strain_terpenes st ON st.terpene_id = t.id
           WHERE st.strain_id = $1
           ORDER BY t.name"#,
    )
    .bind(strain_id)
    .fetch_all(pool)
    .await?;

    let effects = sqlx::query_as::<_, EffectInfo>(
        r#"SELECT e.id, e.name, e.category
           FROM effects e
           JOIN strain_effects se ON se.effect_id = e.id
           WHERE se.strain_id = $1
           ORDER BY e.category, e.name"#,
    )
    .bind(strain_id)
    .fetch_all(pool)
    .await?;

    let primary_photo_key: Option<String> = sqlx::query_scalar(
        "SELECT s3_key FROM strain_photos WHERE strain_id = $1 AND is_primary = true",
    )
    .bind(strain_id)
    .fetch_optional(pool)
    .await?;

    Ok(StrainDetail {
        strain,
        terpenes,
        effects,
        primary_photo_key,
    })
}

/// Create a new public strain. Returns the created strain ID.
#[allow(clippy::too_many_arguments)]
pub async fn create_strain(
    pool: &PgPool,
    name: &str,
    strain_type: &str,
    thc_percentage: Option<f64>,
    cbd_percentage: Option<f64>,
    description: Option<&str>,
    color: Option<&str>,
    smell: Option<&str>,
    flavor: Option<&str>,
    breeder: Option<&str>,
    lineage: Option<&str>,
    growing_difficulty: Option<&str>,
    flowering_time_days: Option<i16>,
    terpene_ids: &[i16],
    effect_ids: &[i16],
) -> AppResult<Uuid> {
    // Check for duplicate name (case-insensitive)
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM public_strains WHERE LOWER(name) = LOWER($1)",
    )
    .bind(name)
    .fetch_one(pool)
    .await?;

    if existing > 0 {
        return Err(AppError::Conflict(format!(
            "A strain named '{}' already exists in the catalog",
            name
        )));
    }

    let strain_id = Uuid::new_v4();

    sqlx::query(
        r#"INSERT INTO public_strains
           (id, name, type, thc_percentage, cbd_percentage, description,
            color, smell, flavor, breeder, lineage, growing_difficulty,
            flowering_time_days)
           VALUES ($1, $2, $3::strain_type, $4, $5, $6, $7, $8, $9, $10, $11, $12::growing_difficulty, $13)"#,
    )
    .bind(strain_id)
    .bind(name)
    .bind(strain_type)
    .bind(thc_percentage)
    .bind(cbd_percentage)
    .bind(description)
    .bind(color)
    .bind(smell)
    .bind(flavor)
    .bind(breeder)
    .bind(lineage)
    .bind(growing_difficulty)
    .bind(flowering_time_days)
    .execute(pool)
    .await?;

    // Insert terpene associations
    for &terpene_id in terpene_ids {
        sqlx::query("INSERT INTO strain_terpenes (strain_id, terpene_id) VALUES ($1, $2)")
            .bind(strain_id)
            .bind(terpene_id)
            .execute(pool)
            .await?;
    }

    // Insert effect associations
    for &effect_id in effect_ids {
        sqlx::query("INSERT INTO strain_effects (strain_id, effect_id) VALUES ($1, $2)")
            .bind(strain_id)
            .bind(effect_id)
            .execute(pool)
            .await?;
    }

    Ok(strain_id)
}

/// Update a public strain. Creates a revision record for vetting.
/// The edit goes live immediately (post-edit vetting model).
#[allow(clippy::too_many_arguments)]
pub async fn update_strain(
    pool: &PgPool,
    strain_id: Uuid,
    proposed_by: Uuid,
    name: &str,
    strain_type: &str,
    thc_percentage: Option<f64>,
    cbd_percentage: Option<f64>,
    description: Option<&str>,
    color: Option<&str>,
    smell: Option<&str>,
    flavor: Option<&str>,
    breeder: Option<&str>,
    lineage: Option<&str>,
    growing_difficulty: Option<&str>,
    flowering_time_days: Option<i16>,
    terpene_ids: &[i16],
    effect_ids: &[i16],
    change_summary: Option<&str>,
) -> AppResult<()> {
    // Fetch current state for revision snapshot
    let old_strain = sqlx::query_as::<_, PublicStrain>(
        r#"SELECT id, name, type::text, thc_percentage::float8, cbd_percentage::float8,
                  description, color, smell, flavor, breeder, lineage,
                  growing_difficulty::text, flowering_time_days,
                  average_rating::float8, rating_count, is_active,
                  created_at, updated_at, version
           FROM public_strains WHERE id = $1"#,
    )
    .bind(strain_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Strain not found".into()))?;

    let old_data = serde_json::to_value(&old_strain).map_err(|e| AppError::Internal(e.into()))?;

    if name.to_lowercase() != old_strain.name.to_lowercase() {
        let name_exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM public_strains WHERE LOWER(name) = LOWER($1) AND id != $2",
        )
        .bind(name)
        .bind(strain_id)
        .fetch_one(pool)
        .await?;

        if name_exists > 0 {
            return Err(AppError::Conflict(format!(
                "A strain named '{}' already exists in the catalog",
                name
            )));
        }
    }

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
    .bind(thc_percentage)
    .bind(cbd_percentage)
    .bind(description)
    .bind(color)
    .bind(smell)
    .bind(flavor)
    .bind(breeder)
    .bind(lineage)
    .bind(growing_difficulty)
    .bind(flowering_time_days)
    .bind(strain_id)
    .execute(pool)
    .await?;

    let new_strain = sqlx::query_as::<_, PublicStrain>(
        r#"SELECT id, name, type::text, thc_percentage::float8, cbd_percentage::float8,
                  description, color, smell, flavor, breeder, lineage,
                  growing_difficulty::text, flowering_time_days,
                  average_rating::float8, rating_count, is_active,
                  created_at, updated_at, version
           FROM public_strains WHERE id = $1"#,
    )
    .bind(strain_id)
    .fetch_one(pool)
    .await?;

    let new_data = serde_json::to_value(&new_strain).map_err(|e| AppError::Internal(e.into()))?;

    // Create revision record (pending vetting)
    sqlx::query(
        r#"INSERT INTO strain_revisions
           (strain_id, proposed_by, change_summary, old_data, new_data, status)
           VALUES ($1, $2, $3, $4, $5, 'pending')"#,
    )
    .bind(strain_id)
    .bind(proposed_by)
    .bind(change_summary)
    .bind(&old_data)
    .bind(&new_data)
    .execute(pool)
    .await?;

    // Replace terpene associations
    sqlx::query("DELETE FROM strain_terpenes WHERE strain_id = $1")
        .bind(strain_id)
        .execute(pool)
        .await?;

    for &terpene_id in terpene_ids {
        sqlx::query("INSERT INTO strain_terpenes (strain_id, terpene_id) VALUES ($1, $2)")
            .bind(strain_id)
            .bind(terpene_id)
            .execute(pool)
            .await?;
    }

    // Replace effect associations
    sqlx::query("DELETE FROM strain_effects WHERE strain_id = $1")
        .bind(strain_id)
        .execute(pool)
        .await?;

    for &effect_id in effect_ids {
        sqlx::query("INSERT INTO strain_effects (strain_id, effect_id) VALUES ($1, $2)")
            .bind(strain_id)
            .bind(effect_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

/// Soft-delete a strain (deactivate).
pub async fn deactivate_strain(pool: &PgPool, strain_id: Uuid) -> AppResult<()> {
    let rows = sqlx::query("UPDATE public_strains SET is_active = false WHERE id = $1")
        .bind(strain_id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound("Strain not found".into()));
    }

    Ok(())
}

/// Get all terpenes (for form picklists).
pub async fn list_terpenes(pool: &PgPool) -> AppResult<Vec<TerpeneInfo>> {
    let terpenes = sqlx::query_as::<_, TerpeneInfo>(
        "SELECT id, name, icon, description FROM terpenes ORDER BY name",
    )
    .fetch_all(pool)
    .await?;

    Ok(terpenes)
}

/// Get all effects (for form picklists).
pub async fn list_effects(pool: &PgPool) -> AppResult<Vec<EffectInfo>> {
    let effects = sqlx::query_as::<_, EffectInfo>(
        "SELECT id, name, category FROM effects ORDER BY category, name",
    )
    .fetch_all(pool)
    .await?;

    Ok(effects)
}

/// Parse a comma-separated string of i16 values.
fn parse_comma_separated_i16(s: &str) -> AppResult<Vec<i16>> {
    s.split(',')
        .map(|p| {
            p.trim()
                .parse::<i16>()
                .map_err(|_| AppError::BadRequest(format!("Invalid ID: '{}'", p)))
        })
        .collect()
}

