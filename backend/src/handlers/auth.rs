use argon2::{PasswordHasher, PasswordVerifier};
use axum::{extract::State, routing::{get, post}, Json, Router};
use chrono::Utc;
use serde_json::json;
use sha2::Digest;
use uuid::Uuid;

use crate::auth::jwt;
use crate::auth::middleware::AuthUser;
use crate::auth::oauth::{self, OAuthProvider};
use crate::errors::{AppError, AppResult};
use crate::models::{
    LoginRequest, RefreshRequest, RegisterRequest, TokenResponse, UserProfile,
};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/oauth/:provider", get(oauth_redirect))
        .route("/oauth/callback", get(oauth_callback))
        .route("/me", get(profile))
}

/// POST /api/v1/auth/register
async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<TokenResponse>> {
    // Validate email format
    if !req.email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".into()));
    }

    // Validate password strength
    if req.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
    }

    // Check for existing user
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE email = $1",
    )
    .bind(&req.email.to_lowercase())
    .fetch_one(&state.pool)
    .await?;

    if existing > 0 {
        return Err(AppError::Conflict("A user with this email already exists".into()));
    }

    // Hash password with argon2id
    let salt = argon2::password_hash::SaltString::generate(&mut rand::thread_rng());
    let argon2_config = argon2::Argon2::default();
    let password_hash = argon2_config
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Password hashing failed: {}", e)))?
        .to_string();

    // Insert user
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, date_of_birth, age_verified, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, false, $6, $6)"#,
    )
    .bind(user_id)
    .bind(req.email.to_lowercase())
    .bind(&password_hash)
    .bind(&req.display_name)
    .bind(req.date_of_birth)
    .bind(now)
    .execute(&state.pool)
    .await?;

    // Issue tokens
    let access_token = jwt::create_access_token(user_id, "user", &state.config.jwt_secret)?;
    let (refresh_token, refresh_hash) = jwt::create_refresh_token();

    // Store refresh token
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, device_info, expires_at)
         VALUES ($1, $2, 'api', NOW() + INTERVAL '30 days')",
    )
    .bind(user_id)
    .bind(&refresh_hash)
    .execute(&state.pool)
    .await?;

    let profile = UserProfile {
        id: user_id,
        display_name: req.display_name,
        role: "user".into(),
        created_at: now,
    };

    Ok(Json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".into(),
        expires_in: 900, // 15 minutes
        user: profile,
    }))
}

/// POST /api/v1/auth/login
async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<TokenResponse>> {
    // Look up user by email
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT id, email, password_hash, display_name, role::text, \
                age_verified, date_of_birth, created_at, updated_at \
         FROM users WHERE email = $1 AND deleted_at IS NULL",
    )
    .bind(req.email.to_lowercase())
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized)?;

    // Verify password
    let password_hash = user.password_hash.as_ref()
        .ok_or_else(|| AppError::Unauthorized)?;

    let parsed_hash = argon2::PasswordHash::new(password_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid stored hash: {}", e)))?;

    argon2::Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized)?;

    // Issue tokens
    let access_token = jwt::create_access_token(user.id, &user.role, &state.config.jwt_secret)?;
    let (refresh_token, refresh_hash) = jwt::create_refresh_token();

    // Store refresh token
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, device_info, expires_at)
         VALUES ($1, $2, 'api', NOW() + INTERVAL '30 days')",
    )
    .bind(user.id)
    .bind(&refresh_hash)
    .execute(&state.pool)
    .await?;

    let profile = UserProfile {
        id: user.id,
        display_name: user.display_name,
        role: user.role,
        created_at: user.created_at,
    };

    Ok(Json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".into(),
        expires_in: 900,
        user: profile,
    }))
}

/// POST /api/v1/auth/refresh
async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<TokenResponse>> {
    // Hash the provided refresh token
    let hash = {
        use sha2::Digest;
        let digest = sha2::Sha256::digest(req.refresh_token.as_bytes());
        format!("{:x}", digest)
    };

    // Look up in database
    let row = sqlx::query_as::<_, (Uuid, String, String, String, chrono::DateTime<Utc>)>(
        r#"SELECT rt.user_id, u.role, u.display_name, u.email, u.created_at
           FROM refresh_tokens rt
           JOIN users u ON u.id = rt.user_id
           WHERE rt.token_hash = $1
             AND rt.revoked_at IS NULL
             AND rt.expires_at > NOW()"#,
    )
    .bind(&hash)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized)?;

    let (user_id, role, display_name, _email, created_at) = row;

    // Revoke old refresh token
    sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE token_hash = $1")
        .bind(&hash)
        .execute(&state.pool)
        .await?;

    // Issue new tokens
    let access_token = jwt::create_access_token(user_id, &role, &state.config.jwt_secret)?;
    let (new_refresh_token, new_hash) = jwt::create_refresh_token();

    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, device_info, expires_at)
         VALUES ($1, $2, 'api', NOW() + INTERVAL '30 days')",
    )
    .bind(user_id)
    .bind(&new_hash)
    .execute(&state.pool)
    .await?;

    let profile = UserProfile {
        id: user_id,
        display_name,
        role,
        created_at,
    };

    Ok(Json(TokenResponse {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".into(),
        expires_in: 900,
        user: profile,
    }))
}

/// POST /api/v1/auth/logout
async fn logout(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let hash = {
        use sha2::Digest;
        let digest = sha2::Sha256::digest(req.refresh_token.as_bytes());
        format!("{:x}", digest)
    };

    sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE token_hash = $1")
        .bind(&hash)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "message": "Logged out successfully" })))
}

/// GET /api/v1/auth/oauth/:provider
///
/// Redirects the user to the OAuth provider's authorization page.
async fn oauth_redirect(
    axum::extract::Path(provider_str): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> AppResult<axum::response::Redirect> {
    let provider = parse_provider(&provider_str)?;
    let redirect_uri = format!("{}/api/v1/auth/oauth/callback", state.config.public_url);
    let url = oauth::get_authorization_url(&provider, &redirect_uri, &state.config)?;
    Ok(axum::response::Redirect::temporary(&url))
}

/// GET /api/v1/auth/oauth/callback
///
/// Handles the OAuth provider's callback, exchanges the code for tokens,
/// and redirects to the frontend with tokens as query parameters.
async fn oauth_callback(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<oauth::OAuthCallbackQuery>,
) -> AppResult<axum::response::Redirect> {
    let provider = OAuthProvider::Google;
    let redirect_uri = format!("{}/api/v1/auth/oauth/callback", state.config.public_url);

    let (access_token, refresh_token) =
        oauth::handle_oauth_callback(&provider, &params.code, &redirect_uri, &state.config, &state.pool).await?;

    // Redirect to frontend callback page with tokens
    let frontend_callback = format!(
        "{}/auth/callback?access_token={}&refresh_token={}",
        state.config.cors_origin, access_token, refresh_token
    );

    Ok(axum::response::Redirect::to(&frontend_callback))
}

/// GET /api/v1/auth/me
///
/// Returns the current user's profile with contribution stats.
async fn profile(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT id, email, password_hash, display_name, role::text, \
                age_verified, date_of_birth, created_at, updated_at \
         FROM users WHERE id = $1",
    )
    .bind(auth.user_id)
    .fetch_one(&state.pool)
    .await?;

    // Get contribution stats
    let strain_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM private_strains WHERE user_id = $1",
    )
    .bind(auth.user_id)
    .fetch_one(&state.pool)
    .await?;

    let pushed_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM private_strains WHERE user_id = $1 AND public_strain_id IS NOT NULL",
    )
    .bind(auth.user_id)
    .fetch_one(&state.pool)
    .await?;

    let rating_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM strain_ratings WHERE user_id = $1",
    )
    .bind(auth.user_id)
    .fetch_one(&state.pool)
    .await?;

    let comment_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM comments WHERE user_id = $1 AND is_deleted = false",
    )
    .bind(auth.user_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "display_name": user.display_name,
        "role": user.role,
        "age_verified": user.age_verified,
        "created_at": user.created_at,
        "stats": {
            "private_strains": strain_count,
            "pushed_to_public": pushed_count,
            "ratings": rating_count,
            "comments": comment_count,
        },
    })))
}

fn parse_provider(s: &str) -> AppResult<OAuthProvider> {
    match s.to_lowercase().as_str() {
        "google" => Ok(OAuthProvider::Google),
        "facebook" => Ok(OAuthProvider::Facebook),
        "microsoft" => Ok(OAuthProvider::Microsoft),
        "apple" => Ok(OAuthProvider::Apple),
        _ => Err(AppError::BadRequest(format!("Unknown OAuth provider: {}", s))),
    }
}
