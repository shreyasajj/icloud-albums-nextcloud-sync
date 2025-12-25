use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Config {
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub sync_interval_minutes: u64,
    pub max_concurrent_downloads: usize,
    pub home_assistant_enabled: bool,
    pub home_assistant_url: String,
    pub home_assistant_token: String,
    pub nextcloud_url: String,
    pub nextcloud_username: String,
    pub nextcloud_password: String,
    pub target_folder: String,
    pub apple_id: String,
    pub anisette_url: String,
    pub device_configured: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            sync_interval_minutes: 60,
            max_concurrent_downloads: 5,
            home_assistant_enabled: false,
            home_assistant_url: String::new(),
            home_assistant_token: String::new(),
            nextcloud_url: String::new(),
            nextcloud_username: String::new(),
            nextcloud_password: String::new(),
            target_folder: "/iCloud Albums".to_string(),
            apple_id: String::new(),
            anisette_url: "https://ani.sidestore.io/v3".to_string(),
            device_configured: false,
        }
    }
}
