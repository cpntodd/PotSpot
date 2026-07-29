use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row for the `users` table.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub display_name: String,
    pub role: String,
    pub age_verified: bool,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub avatar_s3_key: Option<String>,
    pub banner_s3_key: Option<String>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Public-facing user profile (no sensitive fields).
#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub display_name: String,
    pub role: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Profile statistics.
#[derive(Debug, Serialize)]
pub struct ProfileStats {
    pub strains_submitted: i64,
    pub strains_in_vault: i64,
    pub comments: i64,
    pub reviews: i64,
    pub saved_strains: i64,
}

/// Request body for updating profile.
#[derive(Debug, Deserialize)]
pub struct ProfileUpdateRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
}

/// Request body for email/password registration.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub date_of_birth: chrono::NaiveDate,
}

/// Request body for email/password login.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Response body containing JWT tokens.
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u32,
    pub user: UserProfile,
}

/// Request body for refreshing an access token.
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
