use sqlx::PgPool;

use crate::config::Config;

/// Shared application state, accessible by all handlers via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config) -> Self {
        Self { pool, config }
    }
}
