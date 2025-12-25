use super::WebDAVClient;
use anyhow::{Context, Result};
use tracing::{debug, info, instrument};

/// High-level Nextcloud client
#[derive(Debug, Clone)]
pub struct NextcloudClient {
    webdav: WebDAVClient,
}

impl NextcloudClient {
    /// Create a new Nextcloud client
    pub fn new(base_url: String, username: String, password: String) -> Self {
        let webdav = WebDAVClient::new(base_url, username, password);
        Self { webdav }
    }

    /// Create an album folder structure
    /// Creates: target_folder/album_name/
    #[instrument(skip(self))]
    pub async fn create_album_folder(&self, target_folder: &str, album_name: &str) -> Result<String> {
        let folder_path = format!("{}/{}", target_folder.trim_end_matches('/'), album_name);

        // Ensure target folder exists
        self.webdav
            .create_folder(target_folder)
            .await
            .context("Failed to create target folder")?;

        // Create album folder
        self.webdav
            .create_folder(&folder_path)
            .await
            .context("Failed to create album folder")?;

        info!("Created album folder: {}", folder_path);
        Ok(folder_path)
    }

    /// Upload a photo to an album folder
    #[instrument(skip(self, content))]
    pub async fn upload_photo(
        &self,
        album_folder: &str,
        filename: &str,
        content: bytes::Bytes,
    ) -> Result<String> {
        let file_path = format!("{}/{}", album_folder.trim_end_matches('/'), filename);

        self.webdav
            .upload_file(&file_path, content)
            .await
            .context("Failed to upload photo")?;

        debug!("Uploaded photo: {}", file_path);
        Ok(file_path)
    }

    /// Delete a photo from an album folder
    #[instrument(skip(self))]
    pub async fn delete_photo(&self, file_path: &str) -> Result<()> {
        self.webdav
            .delete_file(file_path)
            .await
            .context("Failed to delete photo")?;

        debug!("Deleted photo: {}", file_path);
        Ok(())
    }

    /// Check if a file exists
    #[instrument(skip(self))]
    pub async fn file_exists(&self, file_path: &str) -> Result<bool> {
        self.webdav
            .file_exists(file_path)
            .await
    }

    /// List files in an album folder
    #[instrument(skip(self))]
    pub async fn list_album_files(&self, album_folder: &str) -> Result<Vec<String>> {
        self.webdav
            .list_files(album_folder)
            .await
            .context("Failed to list album files")
    }
}
