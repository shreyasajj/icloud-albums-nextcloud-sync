mod api;
mod config;
mod db;
mod icloud;
mod models;
mod nextcloud;
mod sync;
mod utils;

use crate::api::handlers::{AppContext, AppState};
use crate::config::ServerConfig;
use crate::db::{create_pool, run_migrations};
use crate::sync::{SyncScheduler, SyncService};
use std::sync::Arc;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let config = ServerConfig::from_env();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting iCloud Albums Nextcloud Sync Service");
    info!("Configuration: {:?}", config);

    // Create database connection pool
    info!("Connecting to database...");
    let pool = create_pool(&config.database_url).await?;
    info!("Database connected");

    // Run migrations
    info!("Running database migrations...");
    run_migrations(&pool).await?;
    info!("Migrations completed");

    // Create sync service
    let sync_service = Arc::new(SyncService::new(pool.clone()));

    // Load configuration from database
    if let Err(e) = sync_service.load_config().await {
        error!("Failed to load configuration: {}", e);
    }

    // Create app context
    let app_context = Arc::new(AppContext {
        sync_service: sync_service.clone(),
    });

    // Create router
    let app = api::create_router(app_context);

    // Start sync scheduler in background
    let scheduler = SyncScheduler::new(sync_service.clone());
    tokio::spawn(async move {
        scheduler.start().await;
    });

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.address()).await?;
    info!("Server listening on {}", config.address());

    axum::serve(listener, app).await?;

    Ok(())
}
