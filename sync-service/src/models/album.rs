use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum AlbumStatus {
    Pending,
    Approved,
    Rejected,
    Syncing,
    Synced,
}

impl std::fmt::Display for AlbumStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlbumStatus::Pending => write!(f, "pending"),
            AlbumStatus::Approved => write!(f, "approved"),
            AlbumStatus::Rejected => write!(f, "rejected"),
            AlbumStatus::Syncing => write!(f, "syncing"),
            AlbumStatus::Synced => write!(f, "synced"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Album {
    pub id: i32,
    pub album_id: String,
    pub name: String,
    pub owner: Option<String>,
    pub status: AlbumStatus,
    pub nextcloud_folder: Option<String>,
    pub sync_enabled: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAlbum {
    pub album_id: String,
    pub name: String,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAlbum {
    pub name: Option<String>,
    pub status: Option<AlbumStatus>,
    pub nextcloud_folder: Option<String>,
    pub sync_enabled: Option<bool>,
}
