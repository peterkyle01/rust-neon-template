mod config;
mod error;
mod models;
mod routes;
mod services;

use std::sync::Arc;

use tracing_subscriber::EnvFilter;

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
    let app = routes::build_router(config);
    axum::serve(listener, app).await?;

    Ok(())
}
