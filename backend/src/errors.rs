use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Unified application error type.
/// All errors are serialized to JSON responses with appropriate HTTP status codes.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Authentication required")]
    Unauthorized,

    #[error("Insufficient permissions")]
    Forbidden,

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    UnprocessableEntity(String),

    #[error("Rate limit exceeded. Try again later.")]
    RateLimited,

    #[error("Age verification required. You must be 18 or older.")]
    AgeRestricted,

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", self.to_string()),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND", self.to_string()),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT", self.to_string()),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", self.to_string()),
            AppError::UnprocessableEntity(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "UNPROCESSABLE_ENTITY", self.to_string())
            }
            AppError::RateLimited => {
                (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED", self.to_string())
            }
            AppError::AgeRestricted => {
                (StatusCode::FORBIDDEN, "AGE_RESTRICTED", self.to_string())
            }
            AppError::Internal(_) => {
                tracing::error!(error = %self, "Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "An unexpected error occurred".to_string(),
                )
            }
            AppError::Database(e) => {
                tracing::error!(error = %e, "Database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "A database error occurred".to_string(),
                )
            }
            AppError::Jwt(_) => {
                (StatusCode::UNAUTHORIZED, "INVALID_TOKEN", self.to_string())
            }
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}

/// Convenience type alias for handler results.
pub type AppResult<T> = Result<T, AppError>;
