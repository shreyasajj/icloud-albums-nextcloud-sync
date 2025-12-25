use super::handlers::*;
use axum::{
    routing::{get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // Authentication routes
        .route("/api/auth/login", post(login))
        .route("/api/auth/status", get(auth_status))
        // Config routes
        .route("/api/config", get(get_config).put(update_config))
        // Album routes
        .route("/api/albums", get(list_albums))
        .route("/api/albums/discover-all", post(discover_all_albums))
        .route("/api/albums/:id", get(get_album))
        .route("/api/albums/:id/approve", post(approve_album))
        .route("/api/albums/:id/reject", post(reject_album))
        .route("/api/albums/:id/sync-toggle", put(toggle_album_sync))
        .route("/api/albums/:id/sync", post(sync_album))
        // Sync routes
        .route("/api/sync/all", post(sync_all))
        .route("/api/sync/history", get(get_sync_history))
        .route("/api/sync/history/:id", get(get_album_sync_history))
        // Add CORS layer
        .layer(CorsLayer::permissive())
        .with_state(state)
}
