// Rate limiting middleware using a simple token-bucket algorithm.
// Tracks request counts per IP in an in-memory concurrent map.
// Cleans up expired entries periodically.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::Instant;

use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Rate limit configuration for different endpoint categories.
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Default requests per window
    pub default_limit: u32,
    /// Stricter limit for auth endpoints (login, register)
    pub auth_limit: u32,
    /// Window duration in seconds
    pub window_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            default_limit: 100,
            auth_limit: 5,
            window_secs: 60,
        }
    }
}

struct Bucket {
    tokens: u32,
    last_refill: Instant,
}

/// Global rate limiter state.
static RATE_LIMITER: std::sync::LazyLock<Mutex<HashMap<IpAddr, HashMap<String, Bucket>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Apply rate limiting middleware.
/// Uses the request path prefix to determine which limit to apply.
/// Paths starting with /api/v1/auth get the stricter auth limit.
pub async fn rate_limit(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, Response> {
    let ip = addr.ip();
    let path = req.uri().path();
    let config = RateLimitConfig::default();

    // Determine the bucket key based on path
    let bucket_key = if path.starts_with("/api/v1/auth") {
        "auth"
    } else if path.starts_with("/api/v1/admin") {
        "admin"
    } else {
        "default"
    };

    let limit = match bucket_key {
        "auth" => config.auth_limit,
        _ => config.default_limit,
    };

    let allowed = {
        let mut map = RATE_LIMITER.lock().unwrap();

        // Clean up entries older than 2x the window
        let now = Instant::now();
        map.retain(|_, buckets| {
            buckets.values().any(|b| now.duration_since(b.last_refill).as_secs() < config.window_secs * 2)
        });

        let ip_buckets = map.entry(ip).or_default();
        let bucket = ip_buckets.entry(bucket_key.to_string()).or_insert_with(|| Bucket {
            tokens: limit,
            last_refill: Instant::now(),
        });

        // Refill tokens if window has elapsed
        let elapsed = now.duration_since(bucket.last_refill).as_secs();
        if elapsed >= config.window_secs {
            bucket.tokens = limit;
            bucket.last_refill = now;
        }

        if bucket.tokens > 0 {
            bucket.tokens -= 1;
            true
        } else {
            false
        }
    };

    if allowed {
        Ok(next.run(req).await)
    } else {
        let body = json!({
            "error": {
                "code": "RATE_LIMITED",
                "message": "Too many requests. Please try again later.",
            }
        });
        let response = (
            StatusCode::TOO_MANY_REQUESTS,
            [("Retry-After", "60")],
            axum::Json(body),
        );
        Err(response.into_response())
    }
}
