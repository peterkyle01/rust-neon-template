use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub auth_url: String,
    pub data_api_url: String,
    pub port: u16,
    pub host: String,
}

impl Config {
    /// Load configuration from environment variables.
    /// Uses `.env` files if present (via dotenvy).
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let config = Config {
            auth_url: std::env::var("AUTH_URL").expect("AUTH_URL must be set"),
            data_api_url: std::env::var("DATA_API_URL").expect("DATA_API_URL must be set"),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
        };

        Ok(config)
    }
}
