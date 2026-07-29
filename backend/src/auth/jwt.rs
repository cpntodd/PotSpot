use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

/// JWT claims embedded in access tokens.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject -- the user's UUID.
    pub sub: String,
    /// User's display role.
    pub role: String,
    /// Issued at (Unix timestamp).
    pub iat: usize,
    /// Expiration (Unix timestamp).
    pub exp: usize,
    /// Unique token ID for revocation support.
    pub jti: String,
}

/// Create a short-lived access token (15 minutes).
pub fn create_access_token(
    user_id: Uuid,
    role: &str,
    secret: &str,
) -> AppResult<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + Duration::minutes(15)).timestamp() as usize,
        jti: Uuid::new_v4().to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Jwt(e))
}

/// Create a long-lived opaque refresh token (30 days).
/// The actual token is a random UUID; a SHA-256 hash is stored in the database.
pub fn create_refresh_token() -> (String, String) {
    let token = Uuid::new_v4().to_string();
    let hash = sha2::Sha256::digest(token.as_bytes());
    let hash_hex = format!("{:x}", hash);
    (token, hash_hex)
}

/// Validate an access token and return the claims.
pub fn validate_access_token(token: &str, secret: &str) -> AppResult<Claims> {
    let mut validation = Validation::default();
    // We don't validate `exp` manually; the library does it.
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| AppError::Jwt(e))?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_validate_access_token() {
        let user_id = Uuid::new_v4();
        let secret = "test-secret-key-for-unit-tests";
        let token = create_access_token(user_id, "user", secret).unwrap();
        let claims = validate_access_token(&token, secret).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.role, "user");
    }

    #[test]
    fn test_refresh_token_is_unique() {
        let (t1, h1) = create_refresh_token();
        let (t2, h2) = create_refresh_token();
        assert_ne!(t1, t2);
        assert_ne!(h1, h2);
    }
}
