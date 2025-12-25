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
use crate::icloud::{AuthManager, ICloudClient, MacDeviceConfig};
use crate::sync::{SyncScheduler, SyncService};
use std::sync::Arc;
use tracing::{info, error, warn};
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

    info!("Starting iCloud Albums Nextcloud Sync Service (rustpush edition)");
    info!("Configuration: {:?}", config);

    // Create database connection pool
    info!("Connecting to database...");
    let pool = create_pool(&config.database_url).await?;
    info!("Database connected");

    // Run migrations
    info!("Running database migrations...");
    run_migrations(&pool).await?;
    info!("Migrations completed");

    // Initialize device configuration
    info!("Initializing device configuration...");
    let device_config = match MacDeviceConfig::load_from_db(&pool).await? {
        Some(config) => {
            info!("Loaded existing device configuration");
            config
        }
        None => {
            info!("Creating new device configuration");
            let config = MacDeviceConfig::default();
            config.save_to_db(&pool).await?;
            info!("Device UUID: {}", config.device_uuid);
            config
        }
    };

    // Initialize authentication manager
    info!("Initializing authentication manager...");
    let auth_manager = Arc::new(AuthManager::new(pool.clone(), device_config));

    // Try to auto-authenticate from stored credentials
    info!("Checking for stored credentials...");
    match auth_manager.load_and_authenticate().await {
        Ok(true) => {
            info!("Successfully authenticated from stored credentials");
        }
        Ok(false) => {
            warn!("No stored credentials found. Please login via API.");
        }
        Err(e) => {
            warn!("Failed to authenticate from stored credentials: {}", e);
            warn!("Please login via API.");
        }
    }

    // Initialize iCloud client
    info!("Initializing iCloud client...");
    let icloud_client = ICloudClient::new(auth_manager.clone());
    let icloud_client = Arc::new(tokio::sync::RwLock::new(icloud_client));

    // Try to initialize client if authenticated
    if auth_manager.is_authenticated().await {
        info!("Initializing iCloud SharedStream client...");
        match icloud_client.write().await.initialize().await {
            Ok(_) => {
                info!("iCloud client initialized successfully");
            }
            Err(e) => {
                error!("Failed to initialize iCloud client: {}", e);
                warn!("You may need to re-authenticate via API.");
            }
        }
    }

    // Create sync service
    let sync_service = Arc::new(SyncService::new(
        pool.clone(),
        icloud_client.clone(),
        auth_manager.clone(),
    ));

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
    info!("API Documentation:");
    info!("  POST /api/auth/login - Authenticate with Apple ID");
    info!("  GET  /api/auth/status - Check authentication status");
    info!("  POST /api/albums/discover-all - Discover all shared albums");
    info!("  POST /api/albums/:id/approve - Approve an album for syncing");
    info!("  POST /api/albums/:id/sync - Sync an album now");
    info!("  POST /api/sync/all - Sync all approved albums");

    axum::serve(listener, app).await?;

    Ok(())
}
