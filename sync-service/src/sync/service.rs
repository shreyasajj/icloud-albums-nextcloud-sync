use crate::db::{AlbumRepository, FileRepository, SyncHistoryRepository, ConfigRepository, DbPool};
use crate::icloud::ICloudClient;
use crate::nextcloud::NextcloudClient;
use crate::utils::{calculate_sha256, HomeAssistantClient};
use crate::models::{
    Album, AlbumStatus, CreateAlbum, CreateFile, CreateSyncHistory, FileStatus,
    SyncStatus, SyncType, UpdateAlbum, UpdateFile, UpdateSyncHistory, AppConfig,
};
use anyhow::{Context, Result};
use futures::StreamExt;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

/// Core sync service that orchestrates syncing between iCloud and Nextcloud
pub struct SyncService {
    pool: DbPool,
    icloud_client: ICloudClient,
    config: Arc<tokio::sync::RwLock<AppConfig>>,
}

impl SyncService {
    /// Create a new sync service
    pub fn new(pool: DbPool) -> Self {
        let icloud_client = ICloudClient::new();
        let config = Arc::new(tokio::sync::RwLock::new(AppConfig::default()));

        Self {
            pool,
            icloud_client,
            config,
        }
    }

    /// Load configuration from database
    pub async fn load_config(&self) -> Result<()> {
        let config_repo = ConfigRepository::new(self.pool.clone());
        let app_config = config_repo.get_app_config().await?;

        let mut config = self.config.write().await;
        *config = app_config;

        Ok(())
    }

    /// Get current configuration
    pub async fn get_config(&self) -> AppConfig {
        self.config.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, new_config: AppConfig) -> Result<()> {
        let config_repo = ConfigRepository::new(self.pool.clone());
        config_repo.save_app_config(&new_config).await?;

        let mut config = self.config.write().await;
        *config = new_config;

        Ok(())
    }

    /// Discover a new album from iCloud token and add to database as pending
    #[instrument(skip(self))]
    pub async fn discover_album(&self, token: &str) -> Result<Album> {
        info!("Discovering album from token");

        // Fetch album from iCloud
        let icloud_album = self
            .icloud_client
            .fetch_album(token)
            .await
            .context("Failed to fetch album from iCloud")?;

        // Save to database as pending
        let album_repo = AlbumRepository::new(self.pool.clone());

        // Check if album already exists
        if let Some(existing) = album_repo.get_by_album_id(&icloud_album.token).await? {
            info!("Album already exists: {}", existing.name);
            return Ok(existing);
        }

        let create_album = CreateAlbum {
            album_id: icloud_album.token.clone(),
            name: icloud_album.name.clone(),
            owner: icloud_album.owner.clone(),
        };

        let album = album_repo.create(create_album).await?;

        info!("Discovered new album: {} ({})", album.name, album.id);

        // Send notification to Home Assistant if enabled
        let config = self.get_config().await;
        if config.home_assistant_enabled && !config.home_assistant_url.is_empty() {
            let ha_client = HomeAssistantClient::new(
                config.home_assistant_url,
                config.home_assistant_token,
            );

            if let Err(e) = ha_client
                .notify_new_album(&album.name, album.owner.as_deref().unwrap_or("Unknown"))
                .await
            {
                warn!("Failed to send Home Assistant notification: {}", e);
            }
        }

        Ok(album)
    }

    /// Approve an album for syncing
    #[instrument(skip(self))]
    pub async fn approve_album(&self, album_id: i32) -> Result<Album> {
        info!("Approving album {}", album_id);

        let album_repo = AlbumRepository::new(self.pool.clone());

        let update = UpdateAlbum {
            status: Some(AlbumStatus::Approved),
            ..Default::default()
        };

        let album = album_repo.update(album_id, update).await?;

        info!("Album approved: {}", album.name);

        Ok(album)
    }

    /// Reject an album
    #[instrument(skip(self))]
    pub async fn reject_album(&self, album_id: i32) -> Result<Album> {
        info!("Rejecting album {}", album_id);

        let album_repo = AlbumRepository::new(self.pool.clone());

        let update = UpdateAlbum {
            status: Some(AlbumStatus::Rejected),
            sync_enabled: Some(false),
            ..Default::default()
        };

        let album = album_repo.update(album_id, update).await?;

        info!("Album rejected: {}", album.name);

        Ok(album)
    }

    /// Sync a single album
    #[instrument(skip(self))]
    pub async fn sync_album(&self, album_id: i32) -> Result<()> {
        info!("Syncing album {}", album_id);

        let album_repo = AlbumRepository::new(self.pool.clone());
        let file_repo = FileRepository::new(self.pool.clone());
        let sync_history_repo = SyncHistoryRepository::new(self.pool.clone());
        let config = self.get_config().await;

        // Get album
        let album = album_repo
            .get_by_id(album_id)
            .await?
            .context("Album not found")?;

        // Check if album is approved and sync is enabled
        if album.status != AlbumStatus::Approved && album.status != AlbumStatus::Synced {
            anyhow::bail!("Album is not approved for syncing");
        }

        if !album.sync_enabled {
            anyhow::bail!("Sync is disabled for this album");
        }

        // Create sync history entry
        let sync_history = sync_history_repo
            .create(CreateSyncHistory {
                album_id,
                sync_type: if album.last_sync_at.is_some() {
                    SyncType::Incremental
                } else {
                    SyncType::Full
                },
            })
            .await?;

        // Update album status to syncing
        album_repo
            .update(
                album_id,
                UpdateAlbum {
                    status: Some(AlbumStatus::Syncing),
                    ..Default::default()
                },
            )
            .await?;

        // Perform sync
        let sync_result = self
            .perform_sync(&album, &file_repo, &config)
            .await;

        // Update sync history
        match sync_result {
            Ok((added, removed, updated)) => {
                sync_history_repo
                    .update(
                        sync_history.id,
                        UpdateSyncHistory {
                            files_added: Some(added),
                            files_removed: Some(removed),
                            files_updated: Some(updated),
                            status: Some(SyncStatus::Success),
                            error_message: None,
                        },
                    )
                    .await?;

                // Update album status
                album_repo
                    .update(
                        album_id,
                        UpdateAlbum {
                            status: Some(AlbumStatus::Synced),
                            ..Default::default()
                        },
                    )
                    .await?;

                album_repo.update_last_sync(album_id).await?;

                info!(
                    "Sync completed: +{} -{} ~{}",
                    added, removed, updated
                );
            }
            Err(e) => {
                error!("Sync failed: {}", e);

                sync_history_repo
                    .update(
                        sync_history.id,
                        UpdateSyncHistory {
                            status: Some(SyncStatus::Failed),
                            error_message: Some(e.to_string()),
                            ..Default::default()
                        },
                    )
                    .await?;

                // Revert album status
                album_repo
                    .update(
                        album_id,
                        UpdateAlbum {
                            status: Some(AlbumStatus::Approved),
                            ..Default::default()
                        },
                    )
                    .await?;

                return Err(e);
            }
        }

        Ok(())
    }

    /// Perform the actual sync logic
    async fn perform_sync(
        &self,
        album: &Album,
        file_repo: &FileRepository,
        config: &AppConfig,
    ) -> Result<(i32, i32, i32)> {
        let mut files_added = 0;
        let mut files_removed = 0;
        let files_updated = 0;

        // Fetch current album state from iCloud
        let icloud_album = self
            .icloud_client
            .fetch_album(&album.album_id)
            .await
            .context("Failed to fetch album from iCloud")?;

        // Create Nextcloud client
        let nextcloud_client = NextcloudClient::new(
            config.nextcloud_url.clone(),
            config.nextcloud_username.clone(),
            config.nextcloud_password.clone(),
        );

        // Determine folder path
        let folder_path = if let Some(ref folder) = album.nextcloud_folder {
            folder.clone()
        } else {
            let path = nextcloud_client
                .create_album_folder(&config.target_folder, &album.name)
                .await?;

            // Update album with folder path
            let album_repo = AlbumRepository::new(self.pool.clone());
            album_repo
                .update(
                    album.id,
                    UpdateAlbum {
                        nextcloud_folder: Some(path.clone()),
                        ..Default::default()
                    },
                )
                .await?;

            path
        };

        // Get current files in database
        let db_files = file_repo.list_by_album(album.id).await?;
        let db_guids: std::collections::HashSet<_> =
            db_files.iter().filter_map(|f| f.icloud_guid.as_ref()).collect();

        // Process photos from iCloud
        let max_concurrent = config.max_concurrent_downloads;
        let photos_stream = futures::stream::iter(icloud_album.photos.into_iter())
            .map(|photo| {
                let icloud_client = self.icloud_client.clone();
                let nextcloud_client = nextcloud_client.clone();
                let folder_path = folder_path.clone();
                let file_repo = file_repo.clone();
                let album_id = album.id;

                async move {
                    // Check if photo already exists in database
                    if let Some(existing) = file_repo.get_by_icloud_guid(&photo.guid).await? {
                        debug!("Photo already synced: {}", photo.filename);
                        return Ok::<bool, anyhow::Error>(false);
                    }

                    // Download photo from iCloud
                    debug!("Downloading photo: {}", photo.filename);
                    let content = icloud_client
                        .download_photo(&photo.url)
                        .await
                        .context("Failed to download photo")?;

                    // Calculate hash
                    let hash = calculate_sha256(&content);

                    // Check if file with same hash already exists
                    if let Some(_existing) = file_repo
                        .get_by_album_and_hash(album_id, &hash)
                        .await?
                    {
                        debug!("Photo with same hash already exists: {}", photo.filename);
                        return Ok(false);
                    }

                    // Upload to Nextcloud
                    let nextcloud_path = nextcloud_client
                        .upload_photo(&folder_path, &photo.filename, content.clone())
                        .await
                        .context("Failed to upload photo to Nextcloud")?;

                    // Save to database
                    file_repo
                        .create(CreateFile {
                            album_id,
                            file_name: photo.filename.clone(),
                            file_hash: hash,
                            file_size: Some(photo.file_size as i64),
                            icloud_guid: Some(photo.guid.clone()),
                            icloud_url: Some(photo.url.clone()),
                        })
                        .await?;

                    // Update Nextcloud path
                    if let Some(file) = file_repo.get_by_icloud_guid(&photo.guid).await? {
                        file_repo
                            .update(
                                file.id,
                                UpdateFile {
                                    nextcloud_path: Some(nextcloud_path),
                                    status: Some(FileStatus::Synced),
                                    ..Default::default()
                                },
                            )
                            .await?;
                    }

                    info!("Synced photo: {}", photo.filename);

                    Ok(true)
                }
            })
            .buffer_unordered(max_concurrent);

        // Collect results
        let results: Vec<_> = photos_stream.collect().await;

        for result in results {
            match result {
                Ok(true) => files_added += 1,
                Ok(false) => {}
                Err(e) => {
                    error!("Failed to sync photo: {}", e);
                }
            }
        }

        // Remove photos that are no longer in iCloud
        let icloud_guids: Vec<String> = icloud_album
            .photos
            .iter()
            .map(|p| p.guid.clone())
            .collect();

        // Find files to remove
        for db_file in &db_files {
            if let Some(ref guid) = db_file.icloud_guid {
                if !icloud_guids.contains(guid) {
                    // File is in database but not in iCloud, remove from Nextcloud
                    if let Some(ref path) = db_file.nextcloud_path {
                        if let Err(e) = nextcloud_client.delete_photo(path).await {
                            warn!("Failed to delete photo {}: {}", path, e);
                        } else {
                            file_repo.delete(db_file.id).await?;
                            files_removed += 1;
                            info!("Removed photo: {}", path);
                        }
                    }
                }
            }
        }

        Ok((files_added, files_removed, files_updated))
    }

    /// Sync all albums that need syncing
    #[instrument(skip(self))]
    pub async fn sync_all(&self) -> Result<()> {
        info!("Syncing all albums");

        let album_repo = AlbumRepository::new(self.pool.clone());
        let albums = album_repo.list_syncable().await?;

        info!("Found {} albums to sync", albums.len());

        for album in albums {
            info!("Syncing album: {} ({})", album.name, album.id);

            if let Err(e) = self.sync_album(album.id).await {
                error!("Failed to sync album {}: {}", album.id, e);
                // Continue with next album
            }
        }

        info!("All albums synced");

        Ok(())
    }
}
