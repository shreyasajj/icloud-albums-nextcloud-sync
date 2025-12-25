use anyhow::{Context, Result};
use icloud_album_rs::{get_icloud_photos, ICloudResponse, Image};
use tracing::{debug, info, warn};

pub mod client;

pub use client::ICloudClient;

/// Represents an iCloud album with photos
#[derive(Debug, Clone)]
pub struct Album {
    pub token: String,
    pub name: String,
    pub owner: Option<String>,
    pub photos: Vec<Photo>,
}

/// Represents a photo from iCloud
#[derive(Debug, Clone)]
pub struct Photo {
    pub guid: String,
    pub filename: String,
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub file_size: u64,
}

impl Photo {
    /// Create a Photo from icloud-album-rs Image
    pub fn from_icloud_image(image: &Image) -> Result<Self> {
        // Get the best quality derivative (largest)
        let derivative = image
            .derivatives
            .iter()
            .max_by_key(|d| d.width * d.height)
            .context("No derivatives found for image")?;

        // Extract filename from URL or use GUID
        let filename = derivative
            .url
            .split('/')
            .last()
            .unwrap_or(&image.photo_guid)
            .to_string();

        Ok(Self {
            guid: image.photo_guid.clone(),
            filename,
            url: derivative.url.clone(),
            width: derivative.width,
            height: derivative.height,
            file_size: derivative.file_size,
        })
    }
}

/// Parse iCloud album token from URL or return as-is
pub fn parse_album_token(input: &str) -> String {
    // If it's a full URL, extract the token
    if input.starts_with("http") {
        input
            .split('/')
            .last()
            .unwrap_or(input)
            .to_string()
    } else {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_album_token_from_url() {
        let url = "https://share.icloud.com/photos/abc1234defg5678";
        let token = parse_album_token(url);
        assert_eq!(token, "abc1234defg5678");
    }

    #[test]
    fn test_parse_album_token_from_token() {
        let token = "abc1234defg5678";
        let result = parse_album_token(token);
        assert_eq!(result, "abc1234defg5678");
    }
}
