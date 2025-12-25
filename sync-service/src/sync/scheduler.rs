use super::SyncService;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, instrument};

/// Scheduler for periodic album syncing
pub struct SyncScheduler {
    sync_service: Arc<SyncService>,
}

impl SyncScheduler {
    /// Create a new sync scheduler
    pub fn new(sync_service: Arc<SyncService>) -> Self {
        Self { sync_service }
    }

    /// Start the scheduler
    #[instrument(skip(self))]
    pub async fn start(self) {
        info!("Starting sync scheduler");

        loop {
            // Get sync interval from config
            let config = self.sync_service.get_config().await;
            let interval_minutes = config.sync_interval_minutes;

            info!(
                "Scheduling next sync in {} minutes",
                interval_minutes
            );

            let mut tick_interval = interval(Duration::from_secs(interval_minutes * 60));

            // Wait for next tick
            tick_interval.tick().await;

            // Perform sync
            info!("Running scheduled sync");

            if let Err(e) = self.sync_service.sync_all().await {
                error!("Scheduled sync failed: {}", e);
            }
        }
    }
}
