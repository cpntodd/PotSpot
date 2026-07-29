use std::net::SocketAddr;

use axum::Router;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod auth;
mod config;
mod db;
mod errors;
mod handlers;
mod models;
mod s3;
mod services;
mod state;

use config::Config;
use db::run_migrations;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file (silently skip if not found)
    dotenv().ok();

    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse configuration
    let config = Config::from_env()?;

    // Connect to database
    let pool = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await?;

    // Run migrations
    run_migrations(&pool).await?;

    tracing::info!("Database connected and migrations applied");

    // Build shared application state
    let app_state = AppState::new(pool, config.clone());

    // Build the application router
    let app = Router::new()
        .nest("/api/v1", handlers::build_router())
        .with_state(app_state)
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(
                    config
                        .cors_origin
                        .parse::<axum::http::HeaderValue>()
                        .unwrap_or_else(|_| "*".parse().unwrap()),
                )
                .allow_methods(tower_http::cors::Any)
                .allow_headers(vec![
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::CONTENT_TYPE,
                ]),
        )
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::limit::RequestBodyLimitLayer::new(10 * 1024 * 1024)); // 10 MB max body

    // Bind and serve
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!("PotSpot API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
