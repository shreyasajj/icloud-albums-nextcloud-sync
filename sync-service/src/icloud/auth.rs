use super::osconfig::MacDeviceConfig;
use anyhow::{Context, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rustpush::auth::{authenticate_apple, LoginDelegate};
use rustpush::TokenProvider;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Apple ID credentials (stored encrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleCredentials {
    pub apple_id: String,
    pub encrypted_password: String,
    pub encryption_iv: String,
}

/// Authentication manager for Apple ID
pub struct AuthManager {
    pool: PgPool,
    token_provider: Arc<RwLock<Option<Box<dyn TokenProvider>>>>,
    device_config: MacDeviceConfig,
    encryption_key: [u8; 32],
}

impl AuthManager {
    /// Create new auth manager
    pub fn new(pool: PgPool, device_config: MacDeviceConfig) -> Self {
        // In production, load this from a secure keystore or environment
        // For now, generate from a fixed seed (NOT SECURE - change in production!)
        let encryption_key = Self::derive_key_from_env();

        Self {
            pool,
            token_provider: Arc::new(RwLock::new(None)),
            device_config,
            encryption_key,
        }
    }

    /// Derive encryption key from environment (or generate one)
    fn derive_key_from_env() -> [u8; 32] {
        use sha2::{Digest, Sha256};

        let secret = std::env::var("ENCRYPTION_SECRET")
            .unwrap_or_else(|_| "CHANGE_ME_IN_PRODUCTION".to_string());

        let mut hasher = Sha256::new();
        hasher.update(secret.as_bytes());
        let result = hasher.finalize();

        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        key
    }

    /// Encrypt password for storage
    fn encrypt_password(&self, password: &str) -> Result<(String, String)> {
        let cipher = Aes256Gcm::new(&self.encryption_key.into());

        // Generate random IV
        let iv: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&iv);

        let ciphertext = cipher
            .encrypt(nonce, password.as_bytes())
            .context("Failed to encrypt password")?;

        Ok((hex::encode(ciphertext), hex::encode(iv)))
    }

    /// Decrypt password from storage
    fn decrypt_password(&self, encrypted: &str, iv: &str) -> Result<String> {
        let cipher = Aes256Gcm::new(&self.encryption_key.into());

        let ciphertext = hex::decode(encrypted).context("Invalid encrypted password hex")?;
        let iv_bytes = hex::decode(iv).context("Invalid IV hex")?;
        let nonce = Nonce::from_slice(&iv_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .context("Failed to decrypt password")?;

        String::from_utf8(plaintext).context("Invalid UTF-8 in decrypted password")
    }

    /// Authenticate with Apple ID and password
    pub async fn authenticate(&self, apple_id: &str, password: &str) -> Result<()> {
        info!("Authenticating with Apple ID: {}", apple_id);

        // Create anisette provider (using remote anisette service)
        let anisette_url = std::env::var("ANISETTE_URL")
            .unwrap_or_else(|_| "https://ani.sidestore.io/v3".to_string());

        // Authenticate with Apple
        // This will handle MobileMe and IDS delegates
        let delegates = vec![
            LoginDelegate::MobileMe,
            LoginDelegate::IDS { protocol_version: 4 },
        ];

        // Note: rustpush's authenticate_apple needs omnisette for anisette data
        // We'll need to set this up properly
        let token_provider = authenticate_apple(
            apple_id,
            password,
            &self.device_config,
            &anisette_url,
            delegates,
        )
        .await
        .context("Failed to authenticate with Apple")?;

        // Store token provider
        {
            let mut tp = self.token_provider.write().await;
            *tp = Some(Box::new(token_provider));
        }

        // Encrypt and save credentials
        let (encrypted_password, iv) = self.encrypt_password(password)?;

        self.save_credentials(apple_id, &encrypted_password, &iv)
            .await?;

        info!("Successfully authenticated with Apple ID");

        Ok(())
    }

    /// Save credentials to database
    async fn save_credentials(
        &self,
        apple_id: &str,
        encrypted_password: &str,
        iv: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO credentials (apple_id, encrypted_password, encryption_iv, device_uuid, device_name, device_model)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (apple_id)
            DO UPDATE SET
                encrypted_password = $2,
                encryption_iv = $3,
                updated_at = NOW()
            "#,
        )
        .bind(apple_id)
        .bind(encrypted_password)
        .bind(iv)
        .bind(&self.device_config.device_uuid)
        .bind(&self.device_config.device_name)
        .bind(&self.device_config.device_model)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Load credentials from database and re-authenticate
    pub async fn load_and_authenticate(&self) -> Result<bool> {
        debug!("Loading credentials from database");

        let row = sqlx::query_as::<_, (String, String, String)>(
            r#"
            SELECT apple_id, encrypted_password, encryption_iv
            FROM credentials
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some((apple_id, encrypted_password, iv)) = row {
            let password = self.decrypt_password(&encrypted_password, &iv)?;

            match self.authenticate(&apple_id, &password).await {
                Ok(_) => {
                    info!("Successfully re-authenticated from stored credentials");
                    Ok(true)
                }
                Err(e) => {
                    warn!("Failed to re-authenticate: {}", e);
                    Ok(false)
                }
            }
        } else {
            debug!("No credentials found in database");
            Ok(false)
        }
    }

    /// Get token provider (for use in SharedStreamClient)
    pub async fn get_token_provider(&self) -> Result<Box<dyn TokenProvider>> {
        let tp = self.token_provider.read().await;

        if let Some(ref provider) = *tp {
            Ok(provider.clone())
        } else {
            anyhow::bail!("Not authenticated. Please login first.");
        }
    }

    /// Check if authenticated
    pub async fn is_authenticated(&self) -> bool {
        let tp = self.token_provider.read().await;
        tp.is_some()
    }
}
