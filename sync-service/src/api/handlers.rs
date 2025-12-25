use crate::db::{AlbumRepository, SyncHistoryRepository, ConfigRepository};
use crate::models::AppConfig;
use crate::sync::SyncService;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{result_to_response, ApiResponse};

pub type AppState = Arc<AppContext>;

pub struct AppContext {
    pub sync_service: Arc<SyncService>,
}

// Health check
pub async fn health_check() -> impl IntoResponse {
    Json(ApiResponse::success("OK"))
}

// Config endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigUpdate {
    pub config: AppConfig,
}

pub async fn get_config(State(state): State<AppState>) -> Response {
    let result = state.sync_service.get_config().await;
    (StatusCode::OK, Json(ApiResponse::success(result))).into_response()
}

pub async fn update_config(
    State(state): State<AppState>,
    Json(payload): Json<ConfigUpdate>,
) -> Response {
    let result = state.sync_service.update_config(payload.config).await;
    result_to_response(result)
}

// Album endpoints
pub async fn list_albums(State(state): State<AppState>) -> Response {
    let album_repo = AlbumRepository::new(state.sync_service.pool.clone());
    let result = album_repo.list_all().await;
    result_to_response(result)
}

pub async fn get_album(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Response {
    let album_repo = AlbumRepository::new(state.sync_service.pool.clone());
    let result = album_repo.get_by_id(id).await;

    match result {
        Ok(Some(album)) => (StatusCode::OK, Json(ApiResponse::success(album))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Album not found".to_string())),
        )
            .into_response(),
        Err(e) => result_to_response(Err::<(), _>(e.into())),
    }
}

// Authentication endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub apple_id: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthStatus {
    pub authenticated: bool,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Response {
    let result = state.sync_service.authenticate(&payload.apple_id, &payload.password).await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success("Authentication successful")),
        )
            .into_response(),
        Err(e) => result_to_response(Err::<(), _>(e)),
    }
}

pub async fn auth_status(State(state): State<AppState>) -> Response {
    let authenticated = state.sync_service.is_authenticated().await;
    let status = AuthStatus { authenticated };
    (StatusCode::OK, Json(ApiResponse::success(status))).into_response()
}

// Album discovery - automatic discovery of ALL albums
pub async fn discover_all_albums(State(state): State<AppState>) -> Response {
    let result = state.sync_service.discover_all_albums().await;
    result_to_response(result)
}

pub async fn approve_album(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Response {
    let result = state.sync_service.approve_album(id).await;
    result_to_response(result)
}

pub async fn reject_album(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Response {
    let result = state.sync_service.reject_album(id).await;
    result_to_response(result)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToggleSyncRequest {
    pub enabled: bool,
}

pub async fn toggle_album_sync(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<ToggleSyncRequest>,
) -> Response {
    let album_repo = AlbumRepository::new(state.sync_service.pool.clone());

    let result = album_repo
        .update(
            id,
            crate::models::UpdateAlbum {
                sync_enabled: Some(payload.enabled),
                ..Default::default()
            },
        )
        .await;

    result_to_response(result)
}

pub async fn sync_album(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Response {
    let result = state.sync_service.sync_album(id).await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success("Sync completed")),
        )
            .into_response(),
        Err(e) => result_to_response(Err::<(), _>(e)),
    }
}

// Sync endpoints
pub async fn sync_all(State(state): State<AppState>) -> Response {
    let result = state.sync_service.sync_all().await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success("All albums synced")),
        )
            .into_response(),
        Err(e) => result_to_response(Err::<(), _>(e)),
    }
}

pub async fn get_sync_history(State(state): State<AppState>) -> Response {
    let sync_history_repo = SyncHistoryRepository::new(state.sync_service.pool.clone());
    let result = sync_history_repo.list_all(50).await;
    result_to_response(result)
}

pub async fn get_album_sync_history(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Response {
    let sync_history_repo = SyncHistoryRepository::new(state.sync_service.pool.clone());
    let result = sync_history_repo.list_by_album(id, 20).await;
    result_to_response(result)
}
