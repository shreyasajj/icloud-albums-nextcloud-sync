pub mod routes;
pub mod handlers;
pub mod responses;

pub use routes::create_router;
pub use responses::{ApiResponse, ApiError};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

/// Convert Result to HTTP Response
pub fn result_to_response<T: serde::Serialize>(
    result: anyhow::Result<T>,
) -> Response {
    match result {
        Ok(data) => (StatusCode::OK, Json(ApiResponse::success(data))).into_response(),
        Err(e) => {
            let error = ApiError::from_error(&e);
            (error.status_code(), Json(ApiResponse::<()>::error(error.message.clone()))).into_response()
        }
    }
}
