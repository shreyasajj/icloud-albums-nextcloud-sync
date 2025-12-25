use super::auth::AuthManager;
use anyhow::{Context, Result};
use rustpush::sharedstreams::{
    Album as RustPushAlbum, Asset, SharedStreamClient, SharedStreamsChange,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

/// Album from iCloud SharedStreams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub guid: String,
    pub name: String,
    pub owner_email: Option<String>,
    pub owner_name: Option<String>,
    pub asset_count: usize,
}

/// Photo/Asset from iCloud SharedStreams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Photo {
    pub guid: String,
    pub filename: String,
    pub checksum: String,
    pub file_size: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub mime_type: Option<String>,
}

impl From<&RustPushAlbum> for Album {
    fn from(album: &RustPushAlbum) -> Self {
        Self {
            guid: album.guid.clone(),
            name: album.name.clone().unwrap_or_else(|| "Untitled Album".to_string()),
            owner_email: album.owner_email.clone(),
            owner_name: album.owner_full_name.clone(),
            asset_count: 0, // Will be updated when fetching assets
        }
    }
}

impl From<&Asset> for Photo {
    fn from(asset: &Asset) -> Self {
        // Extract filename from derivative URL or use GUID
        let filename = asset
            .derivative_url
            .as_ref()
            .and_then(|url| url.split('/').last())
            .unwrap_or(&asset.guid)
            .to_string();

        Self {
            guid: asset.guid.clone(),
            filename,
            checksum: asset.checksum.clone().unwrap_or_default(),
            file_size: asset.file_size.unwrap_or(0),
            width: asset.width,
            height: asset.height,
            mime_type: asset.mime_type.clone(),
        }
    }
}

/// Client for interacting with iCloud Shared Albums via rustpush
pub struct ICloudClient {
    auth_manager: Arc<AuthManager>,
    client: Option<SharedStreamClient>,
}

impl ICloudClient {
    /// Create a new iCloud client
    pub fn new(auth_manager: Arc<AuthManager>) -> Self {
        Self {
            auth_manager,
            client: None,
        }
    }

    /// Initialize the SharedStreamClient (requires authentication)
    #[instrument(skip(self))]
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing iCloud SharedStream client");

        if !self.auth_manager.is_authenticated().await {
            info!("Not authenticated, attempting to load credentials");
            let success = self.auth_manager.load_and_authenticate().await?;

            if !success {
                anyhow::bail!("Not authenticated. Please login with Apple ID first.");
            }
        }

        let token_provider = self.auth_manager.get_token_provider().await?;

        // Create SharedStreamClient
        self.client = Some(SharedStreamClient::new(token_provider));

        info!("SharedStream client initialized successfully");

        Ok(())
    }

    /// Ensure client is initialized
    fn get_client(&self) -> Result<&SharedStreamClient> {
        self.client
            .as_ref()
            .context("Client not initialized. Call initialize() first.")
    }

    /// Discover all shared albums (automatic discovery!)
    #[instrument(skip(self))]
    pub async fn discover_all_albums(&self) -> Result<Vec<Album>> {
        info!("Discovering all shared albums");

        let client = self.get_client()?;

        // Get changes from iCloud (pass None for initial sync)
        let changes = client
            .get_changes(None)
            .await
            .context("Failed to get album changes from iCloud")?;

        let albums: Vec<Album> = changes
            .albums
            .iter()
            .filter(|a| !a.is_deleted.unwrap_or(false))
            .map(Album::from)
            .collect();

        info!("Discovered {} albums", albums.len());

        for album in &albums {
            debug!("Album: {} ({})", album.name, album.guid);
        }

        Ok(albums)
    }

    /// Get album details and photos
    #[instrument(skip(self))]
    pub async fn get_album_photos(&self, album_guid: &str) -> Result<Vec<Photo>> {
        info!("Fetching photos for album: {}", album_guid);

        let client = self.get_client()?;

        // Get album summary (contains asset list)
        let summary = client
            .get_album_summary(album_guid)
            .await
            .context("Failed to get album summary")?;

        let asset_guids: Vec<String> = summary
            .assets
            .iter()
            .map(|a| a.guid.clone())
            .collect();

        if asset_guids.is_empty() {
            info!("Album has no assets");
            return Ok(vec![]);
        }

        // Get full asset details
        let assets = client
            .get_assets(album_guid, &asset_guids)
            .await
            .context("Failed to get asset details")?;

        let photos: Vec<Photo> = assets.iter().map(Photo::from).collect();

        info!("Found {} photos in album", photos.len());

        Ok(photos)
    }

    /// Download a photo file
    #[instrument(skip(self))]
    pub async fn download_photo(&self, album_guid: &str, asset_guid: &str) -> Result<bytes::Bytes> {
        debug!("Downloading photo: {} from album: {}", asset_guid, album_guid);

        let client = self.get_client()?;

        // Get asset details first to get download URL
        let assets = client
            .get_assets(album_guid, &[asset_guid.to_string()])
            .await
            .context("Failed to get asset for download")?;

        let asset = assets
            .first()
            .context("Asset not found")?;

        // Download using MMCS (Mobile Me Content Server) protocol
        let file_data = client
            .get_file(asset)
            .await
            .context("Failed to download file")?;

        debug!("Downloaded {} bytes", file_data.len());

        Ok(bytes::Bytes::from(file_data))
    }

    /// Subscribe to a shared album using invitation token (optional - for manual adds)
    #[instrument(skip(self))]
    pub async fn subscribe_to_album(&self, token: &str) -> Result<Album> {
        info!("Subscribing to album with token: {}", token);

        let client = self.get_client()?;

        let album = client
            .subscribe_token(token)
            .await
            .context("Failed to subscribe to album")?;

        info!("Successfully subscribed to album: {}", album.name.as_deref().unwrap_or("Unknown"));

        Ok(Album::from(&album))
    }

    /// Poll for album changes (for incremental sync)
    #[instrument(skip(self))]
    pub async fn poll_changes(&self, continuation_token: Option<String>) -> Result<SharedStreamsChange> {
        debug!("Polling for album changes");

        let client = self.get_client()?;

        let changes = client
            .get_changes(continuation_token.as_deref())
            .await
            .context("Failed to poll for changes")?;

        info!(
            "Got {} album updates, {} deleted",
            changes.albums.len(),
            changes.albums.iter().filter(|a| a.is_deleted.unwrap_or(false)).count()
        );

        Ok(changes)
    }
}
