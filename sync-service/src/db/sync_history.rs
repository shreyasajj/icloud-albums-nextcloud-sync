use crate::models::{SyncHistory, CreateSyncHistory, UpdateSyncHistory};
use sqlx::{PgPool, Result};
use chrono::Utc;

pub struct SyncHistoryRepository {
    pool: PgPool,
}

impl SyncHistoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new sync history entry
    pub async fn create(&self, history: CreateSyncHistory) -> Result<SyncHistory> {
        let now = Utc::now();
        let record = sqlx::query_as::<_, SyncHistory>(
            r#"
            INSERT INTO sync_history (album_id, sync_type, started_at)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(history.album_id)
        .bind(history.sync_type)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Get sync history by ID
    pub async fn get_by_id(&self, id: i32) -> Result<Option<SyncHistory>> {
        let history = sqlx::query_as::<_, SyncHistory>(
            r#"
            SELECT * FROM sync_history WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(history)
    }

    /// List sync history for an album
    pub async fn list_by_album(&self, album_id: i32, limit: i64) -> Result<Vec<SyncHistory>> {
        let history = sqlx::query_as::<_, SyncHistory>(
            r#"
            SELECT * FROM sync_history
            WHERE album_id = $1
            ORDER BY started_at DESC
            LIMIT $2
            "#,
        )
        .bind(album_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    /// List all sync history
    pub async fn list_all(&self, limit: i64) -> Result<Vec<SyncHistory>> {
        let history = sqlx::query_as::<_, SyncHistory>(
            r#"
            SELECT * FROM sync_history
            ORDER BY started_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    /// Update sync history
    pub async fn update(&self, id: i32, update: UpdateSyncHistory) -> Result<SyncHistory> {
        let now = Utc::now();

        let mut query = String::from("UPDATE sync_history SET completed_at = $1");
        let mut param_count = 2;

        if update.files_added.is_some() {
            query.push_str(&format!(", files_added = ${}", param_count));
            param_count += 1;
        }
        if update.files_removed.is_some() {
            query.push_str(&format!(", files_removed = ${}", param_count));
            param_count += 1;
        }
        if update.files_updated.is_some() {
            query.push_str(&format!(", files_updated = ${}", param_count));
            param_count += 1;
        }
        if update.status.is_some() {
            query.push_str(&format!(", status = ${}", param_count));
            param_count += 1;
        }
        if update.error_message.is_some() {
            query.push_str(&format!(", error_message = ${}", param_count));
            param_count += 1;
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

        let mut q = sqlx::query_as::<_, SyncHistory>(&query).bind(now);

        if let Some(added) = update.files_added {
            q = q.bind(added);
        }
        if let Some(removed) = update.files_removed {
            q = q.bind(removed);
        }
        if let Some(updated) = update.files_updated {
            q = q.bind(updated);
        }
        if let Some(status) = update.status {
            q = q.bind(status);
        }
        if let Some(error) = update.error_message {
            q = q.bind(error);
        }

        q = q.bind(id);

        let history = q.fetch_one(&self.pool).await?;
        Ok(history)
    }
}
