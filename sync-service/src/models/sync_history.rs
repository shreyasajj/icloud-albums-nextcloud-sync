use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum SyncType {
    Full,
    Incremental,
}

impl std::fmt::Display for SyncType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncType::Full => write!(f, "full"),
            SyncType::Incremental => write!(f, "incremental"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum SyncStatus {
    Success,
    Failed,
    Partial,
    Running,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::Success => write!(f, "success"),
            SyncStatus::Failed => write!(f, "failed"),
            SyncStatus::Partial => write!(f, "partial"),
            SyncStatus::Running => write!(f, "running"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SyncHistory {
    pub id: i32,
    pub album_id: i32,
    pub sync_type: SyncType,
    pub files_added: i32,
    pub files_removed: i32,
    pub files_updated: i32,
    pub status: SyncStatus,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSyncHistory {
    pub album_id: i32,
    pub sync_type: SyncType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSyncHistory {
    pub files_added: Option<i32>,
    pub files_removed: Option<i32>,
    pub files_updated: Option<i32>,
    pub status: Option<SyncStatus>,
    pub error_message: Option<String>,
}
