use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row for the `public_strains` table.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PublicStrain {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub strain_type: String,
    pub thc_percentage: Option<f64>,
    pub cbd_percentage: Option<f64>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub smell: Option<String>,
    pub flavor: Option<String>,
    pub breeder: Option<String>,
    pub lineage: Option<String>,
    pub growing_difficulty: Option<String>,
    pub flowering_time_days: Option<i16>,
    pub average_rating: Option<f64>,
    pub rating_count: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}

/// Summary view of a strain for list/search results.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct StrainSummary {
    pub id: Uuid,
    pub name: String,
    pub strain_type: String,
    pub thc_percentage: Option<f64>,
    pub cbd_percentage: Option<f64>,
    pub average_rating: Option<f64>,
    pub rating_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

/// Full detail view of a strain.
#[derive(Debug, Serialize)]
pub struct StrainDetail {
    #[serde(flatten)]
    pub strain: PublicStrain,
    pub terpenes: Vec<TerpeneInfo>,
    pub effects: Vec<EffectInfo>,
    /// MinIO object key for the primary photo (used to generate presigned URL).
    #[serde(skip_serializing)]
    pub primary_photo_key: Option<String>,
}

/// Terpene information (from seed data).
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TerpeneInfo {
    pub id: i16,
    pub name: String,
    pub icon: String,
    pub description: Option<String>,
}

/// Effect information (from seed data).
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct EffectInfo {
    pub id: i16,
    pub name: String,
    pub category: String,
}

/// Query parameters for strain search/filter.
#[derive(Debug, Deserialize)]
pub struct StrainSearchQuery {
    pub q: Option<String>,
    #[serde(rename = "type")]
    pub strain_type: Option<String>,
    pub terpenes: Option<String>,   // comma-separated IDs
    pub effects: Option<String>,    // comma-separated IDs
    pub thc_min: Option<f64>,
    pub thc_max: Option<f64>,
    pub cbd_min: Option<f64>,
    pub cbd_max: Option<f64>,
    pub rating_min: Option<f64>,
    pub sort: Option<String>,       // "rating", "name", "thc", "newest"
    pub order: Option<String>,      // "asc", "desc"
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// Private strain (user's vault).
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PrivateStrain {
    pub id: Uuid,
    pub user_id: Uuid,
    pub public_strain_id: Option<Uuid>,
    pub name: String,
    pub strain_type: String,
    pub thc_percentage: Option<f64>,
    pub cbd_percentage: Option<f64>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub smell: Option<String>,
    pub flavor: Option<String>,
    pub breeder: Option<String>,
    pub lineage: Option<String>,
    pub growing_difficulty: Option<String>,
    pub flowering_time_days: Option<i16>,
    pub personal_rating: Option<i16>,
    pub personal_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request body for creating/updating a private strain.
#[derive(Debug, Deserialize)]
pub struct PrivateStrainRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub strain_type: String,
    pub thc_percentage: Option<f64>,
    pub cbd_percentage: Option<f64>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub smell: Option<String>,
    pub flavor: Option<String>,
    pub breeder: Option<String>,
    pub lineage: Option<String>,
    pub growing_difficulty: Option<String>,
    pub flowering_time_days: Option<i16>,
    pub personal_rating: Option<i16>,
    pub personal_notes: Option<String>,
}

/// Request body for creating a public strain.
#[derive(Debug, Deserialize)]
pub struct CreateStrainRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub strain_type: String,
    pub thc_percentage: Option<f64>,
    pub cbd_percentage: Option<f64>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub smell: Option<String>,
    pub flavor: Option<String>,
    pub breeder: Option<String>,
    pub lineage: Option<String>,
    pub growing_difficulty: Option<String>,
    pub flowering_time_days: Option<i16>,
    pub terpene_ids: Vec<i16>,
    pub effect_ids: Vec<i16>,
}

/// Request body for updating a public strain.
#[derive(Debug, Deserialize)]
pub struct UpdateStrainRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub strain_type: String,
    pub thc_percentage: Option<f64>,
    pub cbd_percentage: Option<f64>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub smell: Option<String>,
    pub flavor: Option<String>,
    pub breeder: Option<String>,
    pub lineage: Option<String>,
    pub growing_difficulty: Option<String>,
    pub flowering_time_days: Option<i16>,
    pub terpene_ids: Vec<i16>,
    pub effect_ids: Vec<i16>,
    pub change_summary: Option<String>,
}

/// Response for a paginated list of strains.
#[derive(Debug, Serialize)]
pub struct StrainListResponse {
    pub strains: Vec<StrainSummary>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// Request body for rating a strain.
#[derive(Debug, Deserialize)]
pub struct RateStrainRequest {
    pub rating: i16,
}
