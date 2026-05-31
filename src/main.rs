use std::sync::Arc;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_target(false)
        .without_time()
        .init();

    let config = Arc::new(rust_neon_template::config::Config::from_env()?);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("listening on {}", addr);

    let app = rust_neon_template::routes(config);
    axum::serve(listener, app).await?;

    Ok(())
}
