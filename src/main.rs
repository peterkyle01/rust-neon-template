mod config;
mod error;
mod handlers;

use std::sync::Arc;

use axum::Router;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

/// Build the application router, wiring paths to handler functions.
pub fn routes(config: Arc<config::Config>) -> Router {
    Router::new()
        .route(
            "/health",
            axum::routing::get(handlers::health::health_check),
        )
        .nest(
            "/api/v1/auth",
            Router::new()
                .route("/sign-up", axum::routing::post(handlers::auth::sign_up))
                .route("/sign-in", axum::routing::post(handlers::auth::sign_in))
                .route("/sign-out", axum::routing::post(handlers::auth::sign_out)),
        )
        .nest(
            "/api/v1/notes",
            Router::new()
                .route(
                    "/",
                    axum::routing::post(handlers::notes::create_note)
                        .get(handlers::notes::get_my_notes),
                )
                .route(
                    "/{id}",
                    axum::routing::get(handlers::notes::get_note)
                        .patch(handlers::notes::update_note)
                        .delete(handlers::notes::delete_note),
                ),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(config)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialise structured logging.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    // Load configuration from environment variables.
    let config = Arc::new(config::Config::from_env()?);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("listening on {}", addr);

    // Build the router and serve.
    let app = routes(config);
    axum::serve(listener, app).await?;

    Ok(())
}
