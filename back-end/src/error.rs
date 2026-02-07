use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("Email error: {0}")]
    Email(String),

    #[error("Image processing error: {0}")]
    Image(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_id = Uuid::new_v4();

        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!(%error_id, "Database error details: {:#?}", e);
                eprintln!("DATABASE ERROR: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error occurred (error_id: {})", error_id),
                )
            }
            AppError::Auth(ref msg) => {
                tracing::warn!(%error_id, "Authentication error: {}", msg);
                (StatusCode::UNAUTHORIZED, msg.clone())
            }
            AppError::Validation(ref msg) => {
                tracing::warn!(%error_id, "Validation error: {}", msg);
                (StatusCode::BAD_REQUEST, msg.clone())
            }
            AppError::NotFound(ref msg) => {
                tracing::warn!(%error_id, "Not found error: {}", msg);
                (StatusCode::NOT_FOUND, msg.clone())
            }
            AppError::Forbidden(ref msg) => {
                tracing::warn!(%error_id, "Forbidden error: {}", msg);
                (StatusCode::FORBIDDEN, msg.clone())
            }
            AppError::Unauthorized => {
                tracing::warn!(%error_id, "Unauthorized access attempt");
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
            }
            AppError::Internal(ref e) => {
                tracing::error!(%error_id, "Internal error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::Email(ref msg) => {
                tracing::error!(%error_id, "Email error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Email service error".to_string(),
                )
            }
            AppError::Image(ref msg) => {
                tracing::warn!(%error_id, "Image processing error: {}", msg);
                (StatusCode::BAD_REQUEST, msg.clone())
            }
            AppError::BadRequest(ref msg) => {
                tracing::warn!(%error_id, "Bad request: {}", msg);
                (StatusCode::BAD_REQUEST, msg.clone())
            }
            AppError::Conflict(ref msg) => {
                tracing::warn!(%error_id, "Conflict error: {}", msg);
                (StatusCode::CONFLICT, msg.clone())
            }
        };

        let body = Json(json!({
            "error": error_message,
            "error_id": error_id.to_string(),
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
