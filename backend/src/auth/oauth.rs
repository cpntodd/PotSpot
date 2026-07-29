use axum::{extract::Query, response::Redirect, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::config::Config;
use crate::errors::{AppError, AppResult};

/// OAuth provider identifiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Google,
    Facebook,
    Microsoft,
    Apple,
}

/// Query parameters received from the OAuth callback.
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: String,
}

/// Generate the OAuth authorization URL for a provider.
pub fn get_authorization_url(provider: &OAuthProvider, redirect_uri: &str, config: &Config) -> AppResult<String> {
    match provider {
        OAuthProvider::Google => {
            let client_id = config
                .google_client_id
                .as_ref()
                .ok_or_else(|| AppError::BadRequest("Google OAuth is not configured".into()))?;

            let params = [
                ("client_id", client_id.as_str()),
                ("redirect_uri", redirect_uri),
                ("response_type", "code"),
                ("scope", "openid email profile"),
                ("access_type", "offline"),
                ("prompt", "consent"),
            ];

            let url = url::Url::parse_with_params(
                "https://accounts.google.com/o/oauth2/v2/auth",
                &params,
            )
            .map_err(|e| AppError::Internal(e.into()))?;

            Ok(url.to_string())
        }
        // Facebook, Microsoft, Apple -- to be implemented
        _ => Err(AppError::BadRequest(format!(
            "OAuth provider {:?} is not yet implemented",
            provider
        ))),
    }
}

/// Exchange an OAuth authorization code for tokens and user info.
pub async fn handle_oauth_callback(
    provider: &OAuthProvider,
    code: &str,
    redirect_uri: &str,
    config: &Config,
    pool: &PgPool,
) -> AppResult<(String, String)> {
    // Placeholder: implement Google token exchange
    match provider {
        OAuthProvider::Google => {
            let client_id = config
                .google_client_id
                .as_ref()
                .ok_or_else(|| AppError::BadRequest("Google OAuth is not configured".into()))?;
            let client_secret = config
                .google_client_secret
                .as_ref()
                .ok_or_else(|| AppError::BadRequest("Google OAuth is not configured".into()))?;

            let client = reqwest::Client::new();

            // Exchange code for tokens
            let token_response: serde_json::Value = client
                .post("https://oauth2.googleapis.com/token")
                .form(&[
                    ("client_id", client_id.as_str()),
                    ("client_secret", client_secret.as_str()),
                    ("code", code),
                    ("grant_type", "authorization_code"),
                    ("redirect_uri", redirect_uri),
                ])
                .send()
                .await
                .map_err(|e| AppError::Internal(e.into()))?
                .json()
                .await
                .map_err(|e| AppError::Internal(e.into()))?;

            let access_token = token_response["access_token"]
                .as_str()
                .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Missing access_token in OAuth response")))?;

            // Fetch user info
            let user_info: serde_json::Value = client
                .get("https://www.googleapis.com/oauth2/v3/userinfo")
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|e| AppError::Internal(e.into()))?
                .json()
                .await
                .map_err(|e| AppError::Internal(e.into()))?;

            let email = user_info["email"]
                .as_str()
                .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Missing email in userinfo")))?;
            let name = user_info["name"]
                .as_str()
                .unwrap_or("Unknown User");

            // Find or create user
            let user_id = find_or_create_oauth_user(pool, email, name, "google", &user_info).await?;

            // Issue JWT tokens
            let access_token = crate::auth::jwt::create_access_token(user_id, "user", &config.jwt_secret)?;
            let (refresh_token, refresh_hash) = crate::auth::jwt::create_refresh_token();

            // Store refresh token hash
            sqlx::query(
                "INSERT INTO refresh_tokens (user_id, token_hash, device_info, expires_at)
                 VALUES ($1, $2, 'oauth', NOW() + INTERVAL '30 days')",
            )
            .bind(user_id)
            .bind(&refresh_hash)
            .execute(pool)
            .await?;

            Ok((access_token, refresh_token))
        }
        _ => Err(AppError::BadRequest(format!(
            "OAuth provider {:?} is not yet implemented",
            provider
        ))),
    }
}

/// Find an existing user by email or create a new one from OAuth data.
async fn find_or_create_oauth_user(
    pool: &PgPool,
    email: &str,
    name: &str,
    provider: &str,
    user_info: &serde_json::Value,
) -> AppResult<uuid::Uuid> {
    // Check for existing OAuth link
    let provider_user_id = user_info["sub"]
        .as_str()
        .unwrap_or("unknown");

    let existing = sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT user_id FROM user_oauth_accounts WHERE provider = $1 AND provider_user_id = $2",
    )
    .bind(provider)
    .bind(provider_user_id)
    .fetch_optional(pool)
    .await?;

    if let Some(user_id) = existing {
        return Ok(user_id);
    }

    // Check if a user with this email already exists
    let existing_email = sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    let user_id = if let Some(uid) = existing_email {
        // Link OAuth to existing user
        sqlx::query(
            "INSERT INTO user_oauth_accounts (user_id, provider, provider_user_id)
             VALUES ($1, $2, $3)
             ON CONFLICT DO NOTHING",
        )
        .bind(uid)
        .bind(provider)
        .bind(provider_user_id)
        .execute(pool)
        .await?;
        uid
    } else {
        // Create new user
        let uid = uuid::Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, display_name, age_verified, role)
             VALUES ($1, $2, $3, false, 'user')",
        )
        .bind(uid)
        .bind(email)
        .bind(name)
        .execute(pool)
        .await?;

        // Create OAuth link
        sqlx::query(
            "INSERT INTO user_oauth_accounts (user_id, provider, provider_user_id)
             VALUES ($1, $2, $3)",
        )
        .bind(uid)
        .bind(provider)
        .bind(provider_user_id)
        .execute(pool)
        .await?;

        uid
    };

    Ok(user_id)
}
