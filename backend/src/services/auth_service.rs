// Auth service -- business logic for authentication operations.
// Currently, auth logic lives in handlers/auth.rs.
// This module will be populated as logic is extracted from handlers.

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppResult;

/// Check if a user is age-verified (18+).
pub async fn is_age_verified(pool: &PgPool, user_id: Uuid) -> AppResult<bool> {
    let verified = sqlx::query_scalar::<_, bool>(
        "SELECT age_verified FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(verified)
}
