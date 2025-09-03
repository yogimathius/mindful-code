use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("WebAssembly error: {0}")]
    Wasm(String),

    #[error("ML inference error: {0}")]
    MachineLearning(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "A database error occurred".to_string(),
                )
            }
            AppError::Validation(ref message) => (StatusCode::BAD_REQUEST, message.clone()),
            AppError::Authentication(ref message) => (StatusCode::UNAUTHORIZED, message.clone()),
            AppError::Authorization(ref message) => (StatusCode::FORBIDDEN, message.clone()),
            AppError::NotFound(ref message) => (StatusCode::NOT_FOUND, message.clone()),
            AppError::Conflict(ref message) => (StatusCode::CONFLICT, message.clone()),
            AppError::Internal(ref message) => {
                tracing::error!("Internal error: {}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal error occurred".to_string(),
                )
            }
            AppError::RateLimit => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded".to_string(),
            ),
            AppError::BadRequest(ref message) => (StatusCode::BAD_REQUEST, message.clone()),
            AppError::ServiceUnavailable(ref message) => {
                (StatusCode::SERVICE_UNAVAILABLE, message.clone())
            }
            AppError::Jwt(_) => (
                StatusCode::UNAUTHORIZED,
                "Invalid authentication token".to_string(),
            ),
            AppError::Encryption(ref message) => {
                tracing::error!("Encryption error: {}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Encryption operation failed".to_string(),
                )
            }
            AppError::Wasm(ref message) => {
                tracing::error!("WebAssembly error: {}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "WebAssembly plugin error".to_string(),
                )
            }
            AppError::MachineLearning(ref message) => {
                tracing::error!("ML error: {}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Machine learning inference failed".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;