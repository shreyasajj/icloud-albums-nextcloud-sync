use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum FileStatus {
    Pending,
    Synced,
    Deleted,
    Error,
}

impl std::fmt::Display for FileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileStatus::Pending => write!(f, "pending"),
            FileStatus::Synced => write!(f, "synced"),
            FileStatus::Deleted => write!(f, "deleted"),
            FileStatus::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct File {
    pub id: i32,
    pub album_id: i32,
    pub file_name: String,
    pub file_hash: String,
    pub file_size: Option<i64>,
    pub icloud_guid: Option<String>,
    pub icloud_url: Option<String>,
    pub nextcloud_path: Option<String>,
    pub status: FileStatus,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFile {
    pub album_id: i32,
    pub file_name: String,
    pub file_hash: String,
    pub file_size: Option<i64>,
    pub icloud_guid: Option<String>,
    pub icloud_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFile {
    pub file_name: Option<String>,
    pub nextcloud_path: Option<String>,
    pub status: Option<FileStatus>,
    pub last_modified_at: Option<DateTime<Utc>>,
}
