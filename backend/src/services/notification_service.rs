// Notification service -- creates and queries in-app notifications.

use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppResult;

/// A notification as returned to the client.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Notification {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub reference_id: Option<Uuid>,
    pub message: String,
    pub is_read: bool,
    pub created_at: chrono::DateTime<Utc>,
}

/// Trigger a notification (called from handlers after actions).
pub async fn create_notification(
    pool: &PgPool,
    user_id: Uuid,
    notification_type: &str,
    reference_id: Option<Uuid>,
    message: &str,
) -> AppResult<()> {
    // Check if user has this notification type enabled
    let enabled: bool = sqlx::query_scalar(
        r#"SELECT COALESCE(
            (SELECT enabled FROM user_notification_settings
             WHERE user_id = $1 AND notification_type = $2::notification_type),
            true  -- default: enabled
        )"#,
    )
    .bind(user_id)
    .bind(notification_type)
    .fetch_one(pool)
    .await?;

    if !enabled {
        return Ok(());
    }

    sqlx::query(
        r#"INSERT INTO notifications (user_id, type, reference_id, message)
           VALUES ($1, $2::notification_type, $3, $4)"#,
    )
    .bind(user_id)
    .bind(notification_type)
    .bind(reference_id)
    .bind(message)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get notifications for a user (most recent first, paginated).
pub async fn get_notifications(
    pool: &PgPool,
    user_id: Uuid,
    page: i64,
    per_page: i64,
) -> AppResult<(Vec<Notification>, i64)> {
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let offset = (page - 1) * per_page;
    let rows = sqlx::query_as::<_, Notification>(
        r#"SELECT id, type::text AS notification_type, reference_id,
                  message, is_read, created_at
           FROM notifications
           WHERE user_id = $1
           ORDER BY created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(user_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((rows, total))
}

/// Get unread notification count for a user.
pub async fn get_unread_count(pool: &PgPool, user_id: Uuid) -> AppResult<i64> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = false",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// Mark a notification as read.
pub async fn mark_read(pool: &PgPool, user_id: Uuid, notification_id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE notifications SET is_read = true WHERE id = $1 AND user_id = $2")
        .bind(notification_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Mark all notifications as read for a user.
pub async fn mark_all_read(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE notifications SET is_read = true WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Update notification preferences for a user.
pub async fn update_preferences(
    pool: &PgPool,
    user_id: Uuid,
    notification_type: &str,
    enabled: bool,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO user_notification_settings (user_id, notification_type, enabled)
           VALUES ($1, $2::notification_type, $3)
           ON CONFLICT (user_id, notification_type) DO UPDATE SET enabled = $3"#,
    )
    .bind(user_id)
    .bind(notification_type)
    .bind(enabled)
    .execute(pool)
    .await?;

    Ok(())
}
