use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use tracing::{debug, instrument};

/// Client for sending notifications to Home Assistant
#[derive(Debug, Clone)]
pub struct HomeAssistantClient {
    url: String,
    token: String,
    client: Client,
}

impl HomeAssistantClient {
    /// Create a new Home Assistant client
    pub fn new(url: String, token: String) -> Self {
        let client = Client::new();
        Self { url, token, client }
    }

    /// Send a notification webhook to Home Assistant
    #[instrument(skip(self))]
    pub async fn send_notification(
        &self,
        title: &str,
        message: &str,
        data: Option<serde_json::Value>,
    ) -> Result<()> {
        let webhook_url = format!("{}/api/webhook/icloud-album-notification", self.url);
        debug!("Sending notification to Home Assistant: {}", title);

        let payload = json!({
            "title": title,
            "message": message,
            "data": data.unwrap_or(json!({}))
        });

        let response = self
            .client
            .post(&webhook_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&payload)
            .send()
            .await
            .context("Failed to send notification to Home Assistant")?;

        if response.status().is_success() {
            debug!("Notification sent successfully");
            Ok(())
        } else {
            anyhow::bail!(
                "Failed to send notification: HTTP {}",
                response.status()
            );
        }
    }

    /// Notify about new album share
    #[instrument(skip(self))]
    pub async fn notify_new_album(&self, album_name: &str, owner: &str) -> Result<()> {
        self.send_notification(
            "New iCloud Album Share",
            &format!("{} shared an album: {}", owner, album_name),
            Some(json!({
                "album_name": album_name,
                "owner": owner,
                "action": "approve_or_reject"
            })),
        )
        .await
    }
}
