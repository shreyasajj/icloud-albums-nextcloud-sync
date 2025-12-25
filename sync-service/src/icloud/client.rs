use super::{Album, Photo, parse_album_token};
use anyhow::{Context, Result};
use icloud_album_rs::get_icloud_photos;
use tracing::{debug, info, instrument};

/// Client for interacting with iCloud Shared Albums
#[derive(Debug, Clone)]
pub struct ICloudClient {
    // In future, this will include authentication credentials for full iCloud API access
    // For now, we use token-based access via icloud-album-rs
}

impl ICloudClient {
    /// Create a new iCloud client
    pub fn new() -> Self {
        Self {}
    }

    /// Fetch an album and its photos using a shared album token
    #[instrument(skip(self))]
    pub async fn fetch_album(&self, token: &str) -> Result<Album> {
        let clean_token = parse_album_token(token);
        info!("Fetching iCloud album with token: {}", clean_token);

        let response = get_icloud_photos(&clean_token)
            .await
            .context("Failed to fetch iCloud album")?;

        debug!(
            "Fetched album '{}' with {} photos",
            response.metadata.stream_name,
            response.photos.len()
        );

        let photos = response
            .photos
            .iter()
            .filter_map(|img| match Photo::from_icloud_image(img) {
                Ok(photo) => Some(photo),
                Err(e) => {
                    tracing::warn!(
                        "Failed to convert image {}: {}",
                        img.photo_guid,
                        e
                    );
                    None
                }
            })
            .collect();

        Ok(Album {
            token: clean_token,
            name: response.metadata.stream_name.clone(),
            owner: Some(format!(
                "{} {}",
                response.metadata.user_first_name.unwrap_or_default(),
                response.metadata.user_last_name.unwrap_or_default()
            )),
            photos,
        })
    }

    /// Download a photo from iCloud
    #[instrument(skip(self))]
    pub async fn download_photo(&self, url: &str) -> Result<bytes::Bytes> {
        debug!("Downloading photo from: {}", url);

        let response = reqwest::get(url)
            .await
            .context("Failed to download photo")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download photo: HTTP {}", response.status());
        }

        let bytes = response
            .bytes()
            .await
            .context("Failed to read photo bytes")?;

        debug!("Downloaded {} bytes", bytes.len());

        Ok(bytes)
    }
}

impl Default for ICloudClient {
    fn default() -> Self {
        Self::new()
    }
}
