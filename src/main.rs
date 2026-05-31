mod config;
mod handlers;
mod response;

use std::sync::Arc;

use axum::Router;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
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
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .include_headers(false)
                        .level(tracing::Level::INFO),
                )
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .with_state(config)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure logging — show all info-level events in a compact format.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_target(false)
        .without_time()
        .init();

    let config = Arc::new(config::Config::from_env()?);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("listening on {}", addr);

    let app = routes(config);
    axum::serve(listener, app).await?;

    Ok(())
}
