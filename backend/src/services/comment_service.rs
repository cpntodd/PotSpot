// Comment service -- threaded comments with voting.

use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

/// A comment with its vote status for the current user.
#[derive(Debug, Serialize)]
pub struct CommentNode {
    pub id: Uuid,
    pub strain_id: Uuid,
    pub user_id: Uuid,
    pub display_name: String,
    pub body: String,
    pub upvotes: i32,
    pub downvotes: i32,
    pub user_vote: Option<i16>, // 1, -1, or null if not voted
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub is_deleted: bool,
    pub replies: Vec<CommentNode>,
}

/// Get threaded comments for a strain.
/// Returns top-level comments with nested replies.
pub async fn get_comments(
    pool: &PgPool,
    strain_id: Uuid,
    current_user_id: Option<Uuid>,
) -> AppResult<Vec<CommentNode>> {
    // Fetch all comments for the strain in one query
    let rows = sqlx::query_as::<_, CommentRow>(
        r#"SELECT
            c.id, c.strain_id, c.user_id, u.display_name,
            c.body, c.upvotes, c.downvotes,
            c.parent_comment_id,
            c.created_at, c.updated_at, c.is_deleted,
            COALESCE(cv.vote, 0) AS user_vote
           FROM comments c
           JOIN users u ON u.id = c.user_id
           LEFT JOIN comment_votes cv ON cv.comment_id = c.id AND cv.user_id = $2
           WHERE c.strain_id = $1
           ORDER BY c.created_at ASC"#,
    )
    .bind(strain_id)
    .bind(current_user_id)
    .fetch_all(pool)
    .await?;

    // Build tree structure
    Ok(build_tree(rows))
}

#[derive(Debug, sqlx::FromRow)]
struct CommentRow {
    id: Uuid,
    strain_id: Uuid,
    user_id: Uuid,
    display_name: String,
    body: String,
    upvotes: i32,
    downvotes: i32,
    parent_comment_id: Option<Uuid>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    is_deleted: bool,
    user_vote: Option<i16>,
}

fn build_tree(rows: Vec<CommentRow>) -> Vec<CommentNode> {
    use std::collections::HashMap;

    // Track parent relationships: child_id -> parent_id
    let mut parents: HashMap<Uuid, Option<Uuid>> = HashMap::new();
    let mut nodes: HashMap<Uuid, CommentNode> = HashMap::new();

    for r in rows {
        parents.insert(r.id, r.parent_comment_id);
        let node = CommentNode {
            id: r.id,
            strain_id: r.strain_id,
            user_id: r.user_id,
            display_name: r.display_name,
            body: if r.is_deleted { "[deleted]".into() } else { r.body },
            upvotes: r.upvotes,
            downvotes: r.downvotes,
            user_vote: if r.user_vote == Some(0) { None } else { r.user_vote },
            created_at: r.created_at,
            updated_at: r.updated_at,
            is_deleted: r.is_deleted,
            replies: vec![],
        };
        nodes.insert(r.id, node);
    }

    // Collect child-parent pairs, then attach children to parents
    let mut children: Vec<(Uuid, Uuid)> = vec![]; // (child_id, parent_id)
    let mut roots: Vec<Uuid> = vec![];

    for (id, parent_id) in &parents {
        match parent_id {
            Some(pid) => children.push((*id, *pid)),
            None => roots.push(*id),
        }
    }

    // Attach children to their parents
    for (child_id, parent_id) in children {
        if let Some(child_node) = nodes.remove(&child_id) {
            if let Some(parent_node) = nodes.get_mut(&parent_id) {
                parent_node.replies.push(child_node);
            }
        }
    }

    // Return root nodes (sorted by creation time)
    let mut result: Vec<CommentNode> = roots
        .into_iter()
        .filter_map(|id| nodes.remove(&id))
        .collect();

    result.sort_by_key(|n| n.created_at);
    result
}

/// Post a new comment (top-level or reply).
pub async fn post_comment(
    pool: &PgPool,
    strain_id: Uuid,
    user_id: Uuid,
    parent_comment_id: Option<Uuid>,
    body: &str,
) -> AppResult<Uuid> {
    // Validate strain exists
    let strain_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM public_strains WHERE id = $1 AND is_active = true)",
    )
    .bind(strain_id)
    .fetch_one(pool)
    .await?;

    if !strain_exists {
        return Err(AppError::NotFound("Strain not found".into()));
    }

    // If replying, validate parent exists and belongs to same strain
    if let Some(parent_id) = parent_comment_id {
        let parent_strain = sqlx::query_scalar::<_, Uuid>(
            "SELECT strain_id FROM comments WHERE id = $1",
        )
        .bind(parent_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Parent comment not found".into()))?;

        if parent_strain != strain_id {
            return Err(AppError::BadRequest(
                "Parent comment does not belong to this strain".into(),
            ));
        }
    }

    let comment_id = Uuid::new_v4();

    sqlx::query(
        r#"INSERT INTO comments (id, strain_id, user_id, parent_comment_id, body)
           VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(comment_id)
    .bind(strain_id)
    .bind(user_id)
    .bind(parent_comment_id)
    .bind(body)
    .execute(pool)
    .await?;

    Ok(comment_id)
}

/// Edit the body of a comment (must be owned by the user).
pub async fn edit_comment(
    pool: &PgPool,
    user_id: Uuid,
    comment_id: Uuid,
    body: &str,
) -> AppResult<()> {
    let rows = sqlx::query(
        "UPDATE comments SET body = $1 WHERE id = $2 AND user_id = $3 AND is_deleted = false",
    )
    .bind(body)
    .bind(comment_id)
    .bind(user_id)
    .execute(pool)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound("Comment not found or access denied".into()));
    }

    Ok(())
}

/// Soft-delete a comment (must be owned by the user).
pub async fn delete_comment(
    pool: &PgPool,
    user_id: Uuid,
    comment_id: Uuid,
) -> AppResult<()> {
    let rows = sqlx::query(
        "UPDATE comments SET is_deleted = true WHERE id = $1 AND user_id = $2",
    )
    .bind(comment_id)
    .bind(user_id)
    .execute(pool)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound("Comment not found or access denied".into()));
    }

    Ok(())
}

/// Vote on a comment (upvote +1 or downvote -1).
/// Replaces any existing vote by the same user.
pub async fn vote_comment(
    pool: &PgPool,
    user_id: Uuid,
    comment_id: Uuid,
    vote: i16,
) -> AppResult<()> {
    if vote != 1 && vote != -1 {
        return Err(AppError::BadRequest("Vote must be 1 (upvote) or -1 (downvote)".into()));
    }

    // Check if user already voted
    let existing_vote: Option<i16> = sqlx::query_scalar(
        "SELECT vote FROM comment_votes WHERE comment_id = $1 AND user_id = $2",
    )
    .bind(comment_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match existing_vote {
        Some(old_vote) if old_vote == vote => {
            // Same vote -- remove it (toggle off)
            sqlx::query("DELETE FROM comment_votes WHERE comment_id = $1 AND user_id = $2")
                .bind(comment_id)
                .bind(user_id)
                .execute(pool)
                .await?;

            // Adjust counters
            if vote == 1 {
                sqlx::query("UPDATE comments SET upvotes = upvotes - 1 WHERE id = $1")
                    .bind(comment_id)
                    .execute(pool)
                    .await?;
            } else {
                sqlx::query("UPDATE comments SET downvotes = downvotes - 1 WHERE id = $1")
                    .bind(comment_id)
                    .execute(pool)
                    .await?;
            }
        }
        Some(old_vote) => {
            // Changed vote -- update
            sqlx::query(
                "UPDATE comment_votes SET vote = $1 WHERE comment_id = $2 AND user_id = $3",
            )
            .bind(vote)
            .bind(comment_id)
            .bind(user_id)
            .execute(pool)
            .await?;

            // Adjust counters: remove old, add new
            if old_vote == 1 {
                sqlx::query("UPDATE comments SET upvotes = upvotes - 1, downvotes = downvotes + 1 WHERE id = $1")
                    .bind(comment_id)
                    .execute(pool)
                    .await?;
            } else {
                sqlx::query("UPDATE comments SET upvotes = upvotes + 1, downvotes = downvotes - 1 WHERE id = $1")
                    .bind(comment_id)
                    .execute(pool)
                    .await?;
            }
        }
        None => {
            // New vote
            sqlx::query(
                "INSERT INTO comment_votes (comment_id, user_id, vote) VALUES ($1, $2, $3)",
            )
            .bind(comment_id)
            .bind(user_id)
            .bind(vote)
            .execute(pool)
            .await?;

            if vote == 1 {
                sqlx::query("UPDATE comments SET upvotes = upvotes + 1 WHERE id = $1")
                    .bind(comment_id)
                    .execute(pool)
                    .await?;
            } else {
                sqlx::query("UPDATE comments SET downvotes = downvotes + 1 WHERE id = $1")
                    .bind(comment_id)
                    .execute(pool)
                    .await?;
            }
        }
    }

    Ok(())
}

/// Remove a user's vote from a comment.
pub async fn remove_vote(
    pool: &PgPool,
    user_id: Uuid,
    comment_id: Uuid,
) -> AppResult<()> {
    let existing: Option<i16> = sqlx::query_scalar(
        "SELECT vote FROM comment_votes WHERE comment_id = $1 AND user_id = $2",
    )
    .bind(comment_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some(vote) = existing {
        sqlx::query("DELETE FROM comment_votes WHERE comment_id = $1 AND user_id = $2")
            .bind(comment_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        if vote == 1 {
            sqlx::query("UPDATE comments SET upvotes = upvotes - 1 WHERE id = $1")
                .bind(comment_id)
                .execute(pool)
                .await?;
        } else {
            sqlx::query("UPDATE comments SET downvotes = downvotes - 1 WHERE id = $1")
                .bind(comment_id)
                .execute(pool)
                .await?;
        }
    }

    Ok(())
}
