mod config;
mod handlers;
mod response;

use std::sync::Arc;

use axum::Router;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
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
                .on_response(LogOnResponse),
        )
        .with_state(config)
}

#[derive(Clone)]
struct LogOnResponse;

impl<B> tower_http::trace::OnResponse<B> for LogOnResponse {
    fn on_response(
        self,
        response: &axum::http::Response<B>,
        latency: std::time::Duration,
        _span: &tracing::Span,
    ) {
        let status = response.status();
        let ms = latency.as_millis();
        match status.as_u16() {
            200..=399 => tracing::info!(status = status.as_u16(), latency_ms = ms, "ok"),
            400..=499 => tracing::warn!(status = status.as_u16(), latency_ms = ms, "client error"),
            _ => tracing::error!(status = status.as_u16(), latency_ms = ms, "server error"),
        }
    }
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
