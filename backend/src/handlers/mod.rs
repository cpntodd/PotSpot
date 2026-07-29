pub mod auth;
pub mod strains;
pub mod vault;
pub mod comments;
pub mod vetting;
pub mod photos;
pub mod admin;

use axum::Router;

use crate::state::AppState;

/// Build the complete API router tree under `/api/v1`.
pub fn build_router() -> Router<AppState> {
    Router::new()
        // Public auth routes (no middleware)
        .nest("/auth", auth::router())
        // Protected strain routes
        .nest("/strains", strains::router())
        // Protected vault routes
        .nest("/vault", vault::router())
        // Comment routes
        .nest("/comments", comments::router())
        // Vetting routes (vetter+ only)
        .nest("/vetting", vetting::router())
        // Photo routes
        .nest("/photos", photos::router())
        // Admin routes (admin only)
        .nest("/admin", admin::router())
}
