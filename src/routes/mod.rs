pub mod auth;
pub mod health;

use axum::Router;
use std::sync::Arc;

use crate::config::Config;

/// Build the combined application router with all routes registered.
pub fn build_router(config: Arc<Config>) -> Router {
    Router::new()
        .nest("/api/v1/auth", auth::routes())
        .route("/health", axum::routing::get(health::health_check))
        .with_state(config)
}
