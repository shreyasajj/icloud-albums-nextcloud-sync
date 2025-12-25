use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::time::Duration;

pub mod albums;
pub mod files;
pub mod sync_history;
pub mod config;

pub use albums::AlbumRepository;
pub use files::FileRepository;
pub use sync_history::SyncHistoryRepository;
pub use config::ConfigRepository;

pub type DbPool = Pool<Postgres>;

/// Create a new database connection pool
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await
}

/// Run database migrations
pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
}
