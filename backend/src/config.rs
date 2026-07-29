use std::env;

/// Application configuration loaded from environment variables.
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub db_max_connections: u32,
    pub jwt_secret: String,
    pub jwt_refresh_secret: String,
    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub minio_bucket: String,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub facebook_client_id: Option<String>,
    pub facebook_client_secret: Option<String>,
    pub microsoft_client_id: Option<String>,
    pub microsoft_client_secret: Option<String>,
    pub apple_client_id: Option<String>,
    pub apple_client_secret: Option<String>,
    pub host: String,
    pub port: u16,
    pub cors_origin: String,
    /// Publicly accessible URL of this API server (for OAuth redirects).
    /// Defaults to http://localhost:{port} in development.
    pub public_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse()?;
        let cors_origin = env::var("CORS_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:5173".into());

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?,
            db_max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".into())
                .parse()?,
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| anyhow::anyhow!("JWT_SECRET must be set"))?,
            jwt_refresh_secret: env::var("JWT_REFRESH_SECRET")
                .map_err(|_| anyhow::anyhow!("JWT_REFRESH_SECRET must be set"))?,
            minio_endpoint: env::var("MINIO_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".into()),
            minio_access_key: env::var("MINIO_ACCESS_KEY")
                .unwrap_or_else(|_| "minioadmin".into()),
            minio_secret_key: env::var("MINIO_SECRET_KEY")
                .unwrap_or_else(|_| "minioadmin".into()),
            minio_bucket: env::var("MINIO_BUCKET")
                .unwrap_or_else(|_| "potspot-photos".into()),
            google_client_id: env::var("GOOGLE_CLIENT_ID").ok(),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET").ok(),
            facebook_client_id: env::var("FACEBOOK_CLIENT_ID").ok(),
            facebook_client_secret: env::var("FACEBOOK_CLIENT_SECRET").ok(),
            microsoft_client_id: env::var("MICROSOFT_CLIENT_ID").ok(),
            microsoft_client_secret: env::var("MICROSOFT_CLIENT_SECRET").ok(),
            apple_client_id: env::var("APPLE_CLIENT_ID").ok(),
            apple_client_secret: env::var("APPLE_CLIENT_SECRET").ok(),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port,
            cors_origin,
            public_url: env::var("PUBLIC_URL")
                .unwrap_or_else(|_| format!("http://localhost:{}", port)),
        })
    }
}
