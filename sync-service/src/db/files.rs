use crate::models::{File, FileStatus, CreateFile, UpdateFile};
use sqlx::{PgPool, Result};
use chrono::Utc;

pub struct FileRepository {
    pool: PgPool,
}

impl FileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new file
    pub async fn create(&self, file: CreateFile) -> Result<File> {
        let now = Utc::now();
        let record = sqlx::query_as::<_, File>(
            r#"
            INSERT INTO files (album_id, file_name, file_hash, file_size, icloud_guid, icloud_url, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(file.album_id)
        .bind(&file.file_name)
        .bind(&file.file_hash)
        .bind(file.file_size)
        .bind(&file.icloud_guid)
        .bind(&file.icloud_url)
        .bind(FileStatus::Pending)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Get file by ID
    pub async fn get_by_id(&self, id: i32) -> Result<Option<File>> {
        let file = sqlx::query_as::<_, File>(
            r#"
            SELECT * FROM files WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(file)
    }

    /// Get file by iCloud GUID
    pub async fn get_by_icloud_guid(&self, guid: &str) -> Result<Option<File>> {
        let file = sqlx::query_as::<_, File>(
            r#"
            SELECT * FROM files WHERE icloud_guid = $1
            "#,
        )
        .bind(guid)
        .fetch_optional(&self.pool)
        .await?;

        Ok(file)
    }

    /// Get file by album and hash
    pub async fn get_by_album_and_hash(&self, album_id: i32, hash: &str) -> Result<Option<File>> {
        let file = sqlx::query_as::<_, File>(
            r#"
            SELECT * FROM files WHERE album_id = $1 AND file_hash = $2
            "#,
        )
        .bind(album_id)
        .bind(hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(file)
    }

    /// List files for an album
    pub async fn list_by_album(&self, album_id: i32) -> Result<Vec<File>> {
        let files = sqlx::query_as::<_, File>(
            r#"
            SELECT * FROM files WHERE album_id = $1 ORDER BY created_at DESC
            "#,
        )
        .bind(album_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(files)
    }

    /// List files by status for an album
    pub async fn list_by_album_and_status(
        &self,
        album_id: i32,
        status: FileStatus,
    ) -> Result<Vec<File>> {
        let files = sqlx::query_as::<_, File>(
            r#"
            SELECT * FROM files WHERE album_id = $1 AND status = $2 ORDER BY created_at DESC
            "#,
        )
        .bind(album_id)
        .bind(status)
        .fetch_all(&self.pool)
        .await?;

        Ok(files)
    }

    /// Update file
    pub async fn update(&self, id: i32, update: UpdateFile) -> Result<File> {
        let now = Utc::now();

        let mut query = String::from("UPDATE files SET updated_at = $1");
        let mut param_count = 2;

        if update.file_name.is_some() {
            query.push_str(&format!(", file_name = ${}", param_count));
            param_count += 1;
        }
        if update.nextcloud_path.is_some() {
            query.push_str(&format!(", nextcloud_path = ${}", param_count));
            param_count += 1;
        }
        if update.status.is_some() {
            query.push_str(&format!(", status = ${}", param_count));
            param_count += 1;
        }
        if update.last_modified_at.is_some() {
            query.push_str(&format!(", last_modified_at = ${}", param_count));
            param_count += 1;
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

        let mut q = sqlx::query_as::<_, File>(&query).bind(now);

        if let Some(name) = update.file_name {
            q = q.bind(name);
        }
        if let Some(path) = update.nextcloud_path {
            q = q.bind(path);
        }
        if let Some(status) = update.status {
            q = q.bind(status);
        }
        if let Some(modified) = update.last_modified_at {
            q = q.bind(modified);
        }

        q = q.bind(id);

        let file = q.fetch_one(&self.pool).await?;
        Ok(file)
    }

    /// Delete file
    pub async fn delete(&self, id: i32) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM files WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete files not in the given GUID list for an album
    pub async fn delete_not_in_guids(&self, album_id: i32, guids: &[String]) -> Result<i32> {
        let result = sqlx::query(
            r#"
            DELETE FROM files
            WHERE album_id = $1
            AND icloud_guid IS NOT NULL
            AND icloud_guid != ALL($2)
            "#,
        )
        .bind(album_id)
        .bind(guids)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i32)
    }
}
