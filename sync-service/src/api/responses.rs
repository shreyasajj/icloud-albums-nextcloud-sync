use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

#[derive(Debug)]
pub struct ApiError {
    pub message: String,
    pub status: StatusCode,
}

impl ApiError {
    pub fn new(message: String, status: StatusCode) -> Self {
        Self { message, status }
    }

    pub fn from_error(error: &anyhow::Error) -> Self {
        Self {
            message: error.to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        self.status
    }
}
