use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use reqwest::{Client, Response};
use tracing::{debug, instrument};

/// WebDAV client for Nextcloud
#[derive(Debug, Clone)]
pub struct WebDAVClient {
    base_url: String,
    username: String,
    password: String,
    client: Client,
}

impl WebDAVClient {
    /// Create a new WebDAV client
    pub fn new(base_url: String, username: String, password: String) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            username,
            password,
            client,
        }
    }

    /// Get the WebDAV URL for a path
    fn webdav_url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{}/remote.php/dav/files/{}/{}", base, self.username, path)
    }

    /// Get basic auth header value
    fn auth_header(&self) -> String {
        let credentials = format!("{}:{}", self.username, self.password);
        let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());
        format!("Basic {}", encoded)
    }

    /// Create a folder (MKCOL)
    #[instrument(skip(self))]
    pub async fn create_folder(&self, path: &str) -> Result<()> {
        let url = self.webdav_url(path);
        debug!("Creating folder: {}", url);

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"MKCOL")?, &url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .context("Failed to create folder")?;

        // 201 Created or 405 Method Not Allowed (folder already exists)
        if response.status().is_success() || response.status() == 405 {
            debug!("Folder created or already exists: {}", path);
            Ok(())
        } else {
            anyhow::bail!(
                "Failed to create folder {}: HTTP {}",
                path,
                response.status()
            );
        }
    }

    /// Upload a file (PUT)
    #[instrument(skip(self, content))]
    pub async fn upload_file(&self, path: &str, content: bytes::Bytes) -> Result<()> {
        let url = self.webdav_url(path);
        debug!("Uploading file: {} ({} bytes)", url, content.len());

        let response = self
            .client
            .put(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/octet-stream")
            .body(content)
            .send()
            .await
            .context("Failed to upload file")?;

        if response.status().is_success() {
            debug!("File uploaded: {}", path);
            Ok(())
        } else {
            anyhow::bail!(
                "Failed to upload file {}: HTTP {}",
                path,
                response.status()
            );
        }
    }

    /// Delete a file (DELETE)
    #[instrument(skip(self))]
    pub async fn delete_file(&self, path: &str) -> Result<()> {
        let url = self.webdav_url(path);
        debug!("Deleting file: {}", url);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .context("Failed to delete file")?;

        if response.status().is_success() || response.status() == 404 {
            debug!("File deleted or not found: {}", path);
            Ok(())
        } else {
            anyhow::bail!(
                "Failed to delete file {}: HTTP {}",
                path,
                response.status()
            );
        }
    }

    /// Check if a file exists (HEAD)
    #[instrument(skip(self))]
    pub async fn file_exists(&self, path: &str) -> Result<bool> {
        let url = self.webdav_url(path);

        let response = self
            .client
            .head(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .context("Failed to check file existence")?;

        Ok(response.status().is_success())
    }

    /// List files in a directory (PROPFIND)
    #[instrument(skip(self))]
    pub async fn list_files(&self, path: &str) -> Result<Vec<String>> {
        let url = self.webdav_url(path);
        debug!("Listing files: {}", url);

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"PROPFIND")?, &url)
            .header("Authorization", self.auth_header())
            .header("Depth", "1")
            .send()
            .await
            .context("Failed to list files")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to list files in {}: HTTP {}",
                path,
                response.status()
            );
        }

        // Parse XML response (simplified - in production use proper XML parser)
        let body = response.text().await?;

        // Extract file names from WebDAV XML response
        // This is a simplified extraction - production code should use proper XML parsing
        let files: Vec<String> = body
            .lines()
            .filter(|line| line.contains("<d:href>"))
            .filter_map(|line| {
                line.split("<d:href>")
                    .nth(1)?
                    .split("</d:href>")
                    .next()
                    .map(|s| s.to_string())
            })
            .collect();

        Ok(files)
    }
}
