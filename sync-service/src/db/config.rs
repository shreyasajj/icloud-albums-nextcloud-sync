use crate::models::{Config, AppConfig};
use sqlx::{PgPool, Result};
use chrono::Utc;

pub struct ConfigRepository {
    pool: PgPool,
}

impl ConfigRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get config value by key
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let config = sqlx::query_as::<_, Config>(
            r#"
            SELECT * FROM config WHERE key = $1
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(config.map(|c| c.value))
    }

    /// Set config value
    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO config (key, value, updated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (key)
            DO UPDATE SET value = $2, updated_at = $3
            "#,
        )
        .bind(key)
        .bind(value)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all config as AppConfig
    pub async fn get_app_config(&self) -> Result<AppConfig> {
        let mut config = AppConfig::default();

        if let Some(value) = self.get("sync_interval_minutes").await? {
            config.sync_interval_minutes = value.parse().unwrap_or(60);
        }
        if let Some(value) = self.get("max_concurrent_downloads").await? {
            config.max_concurrent_downloads = value.parse().unwrap_or(5);
        }
        if let Some(value) = self.get("home_assistant_enabled").await? {
            config.home_assistant_enabled = value.parse().unwrap_or(false);
        }
        if let Some(value) = self.get("home_assistant_url").await? {
            config.home_assistant_url = value;
        }
        if let Some(value) = self.get("home_assistant_token").await? {
            config.home_assistant_token = value;
        }
        if let Some(value) = self.get("nextcloud_url").await? {
            config.nextcloud_url = value;
        }
        if let Some(value) = self.get("nextcloud_username").await? {
            config.nextcloud_username = value;
        }
        if let Some(value) = self.get("nextcloud_password").await? {
            config.nextcloud_password = value;
        }
        if let Some(value) = self.get("target_folder").await? {
            config.target_folder = value;
        }
        if let Some(value) = self.get("apple_id").await? {
            config.apple_id = value;
        }
        if let Some(value) = self.get("anisette_url").await? {
            config.anisette_url = value;
        }
        if let Some(value) = self.get("device_configured").await? {
            config.device_configured = value.parse().unwrap_or(false);
        }

        Ok(config)
    }

    /// Save AppConfig
    pub async fn save_app_config(&self, config: &AppConfig) -> Result<()> {
        self.set("sync_interval_minutes", &config.sync_interval_minutes.to_string()).await?;
        self.set("max_concurrent_downloads", &config.max_concurrent_downloads.to_string()).await?;
        self.set("home_assistant_enabled", &config.home_assistant_enabled.to_string()).await?;
        self.set("home_assistant_url", &config.home_assistant_url).await?;
        self.set("home_assistant_token", &config.home_assistant_token).await?;
        self.set("nextcloud_url", &config.nextcloud_url).await?;
        self.set("nextcloud_username", &config.nextcloud_username).await?;
        self.set("nextcloud_password", &config.nextcloud_password).await?;
        self.set("target_folder", &config.target_folder).await?;
        self.set("apple_id", &config.apple_id).await?;
        self.set("anisette_url", &config.anisette_url).await?;
        self.set("device_configured", &config.device_configured.to_string()).await?;

        Ok(())
    }
}
