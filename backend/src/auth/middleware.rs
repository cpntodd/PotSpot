use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::jwt::{validate_access_token, Claims};
use crate::errors::AppError;
use crate::state::AppState;

/// User role enum matching the database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Vetter,
    Admin,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::User => "user",
            UserRole::Vetter => "vetter",
            UserRole::Admin => "admin",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "user" => Some(UserRole::User),
            "vetter" => Some(UserRole::Vetter),
            "admin" => Some(UserRole::Admin),
            _ => None,
        }
    }

    /// Check if this role is at least the given level.
    pub fn is_at_least(&self, required: &UserRole) -> bool {
        match (self, required) {
            (UserRole::Admin, _) => true,
            (UserRole::Vetter, UserRole::Vetter | UserRole::User) => true,
            (UserRole::User, UserRole::User) => true,
            _ => false,
        }
    }
}

/// Authenticated user information extracted from the JWT.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: UserRole,
}

/// Axum extractor that requires a valid JWT in the Authorization header.
/// Uses `AppState` to access the JWT secret for validation.
#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        // Must be "Bearer <token>"
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        // Validate the token
        let claims = validate_access_token(token, &state.config.jwt_secret)?;

        // Parse user ID
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized)?;

        // Parse role
        let role = UserRole::from_str(&claims.role)
            .ok_or(AppError::Unauthorized)?;

        Ok(AuthUser { user_id, role })
    }
}

