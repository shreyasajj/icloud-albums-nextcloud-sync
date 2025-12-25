use crate::models::{Album, AlbumStatus, CreateAlbum, UpdateAlbum};
use sqlx::{PgPool, Result};
use chrono::Utc;

pub struct AlbumRepository {
    pool: PgPool,
}

impl AlbumRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new album
    pub async fn create(&self, album: CreateAlbum) -> Result<Album> {
        let now = Utc::now();
        let record = sqlx::query_as::<_, Album>(
            r#"
            INSERT INTO albums (album_id, name, owner, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(&album.album_id)
        .bind(&album.name)
        .bind(&album.owner)
        .bind(AlbumStatus::Pending)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Get album by ID
    pub async fn get_by_id(&self, id: i32) -> Result<Option<Album>> {
        let album = sqlx::query_as::<_, Album>(
            r#"
            SELECT * FROM albums WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(album)
    }

    /// Get album by album_id (external ID)
    pub async fn get_by_album_id(&self, album_id: &str) -> Result<Option<Album>> {
        let album = sqlx::query_as::<_, Album>(
            r#"
            SELECT * FROM albums WHERE album_id = $1
            "#,
        )
        .bind(album_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(album)
    }

    /// List all albums
    pub async fn list_all(&self) -> Result<Vec<Album>> {
        let albums = sqlx::query_as::<_, Album>(
            r#"
            SELECT * FROM albums ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(albums)
    }

    /// List albums by status
    pub async fn list_by_status(&self, status: AlbumStatus) -> Result<Vec<Album>> {
        let albums = sqlx::query_as::<_, Album>(
            r#"
            SELECT * FROM albums WHERE status = $1 ORDER BY created_at DESC
            "#,
        )
        .bind(status)
        .fetch_all(&self.pool)
        .await?;

        Ok(albums)
    }

    /// List albums that need syncing
    pub async fn list_syncable(&self) -> Result<Vec<Album>> {
        let albums = sqlx::query_as::<_, Album>(
            r#"
            SELECT * FROM albums
            WHERE sync_enabled = true
            AND status IN ('approved', 'synced')
            ORDER BY last_sync_at ASC NULLS FIRST
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(albums)
    }

    /// Update album
    pub async fn update(&self, id: i32, update: UpdateAlbum) -> Result<Album> {
        let now = Utc::now();

        // Build dynamic update query based on provided fields
        let mut query = String::from("UPDATE albums SET updated_at = $1");
        let mut param_count = 2;

        if update.name.is_some() {
            query.push_str(&format!(", name = ${}", param_count));
            param_count += 1;
        }
        if update.status.is_some() {
            query.push_str(&format!(", status = ${}", param_count));
            param_count += 1;
        }
        if update.nextcloud_folder.is_some() {
            query.push_str(&format!(", nextcloud_folder = ${}", param_count));
            param_count += 1;
        }
        if update.sync_enabled.is_some() {
            query.push_str(&format!(", sync_enabled = ${}", param_count));
            param_count += 1;
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

        let mut q = sqlx::query_as::<_, Album>(&query).bind(now);

        if let Some(name) = update.name {
            q = q.bind(name);
        }
        if let Some(status) = update.status {
            q = q.bind(status);
        }
        if let Some(folder) = update.nextcloud_folder {
            q = q.bind(folder);
        }
        if let Some(enabled) = update.sync_enabled {
            q = q.bind(enabled);
        }

        q = q.bind(id);

        let album = q.fetch_one(&self.pool).await?;
        Ok(album)
    }

    /// Update last sync time
    pub async fn update_last_sync(&self, id: i32) -> Result<()> {
        let now = Utc::now();
        sqlx::query(
            r#"
            UPDATE albums SET last_sync_at = $1, updated_at = $1 WHERE id = $2
            "#,
        )
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete album
    pub async fn delete(&self, id: i32) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM albums WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
