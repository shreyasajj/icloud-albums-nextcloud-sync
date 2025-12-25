use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub log_level: String,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
