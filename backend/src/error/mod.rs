pub mod codes;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    /// User-facing validation / business-rule message — safe to expose.
    #[error("{0}")]
    Validation(String),
    /// Database error — logged server-side, generic message to client.
    #[error("Database error: {0}")]
    Database(Box<surrealdb::Error>),
    /// Internal error — logged server-side, generic message to client.
    /// Use `Validation` for user-facing messages instead.
    #[error("Internal: {0}")]
    Internal(String),
}

impl From<surrealdb::Error> for AppError {
    fn from(e: surrealdb::Error) -> Self {
        Self::Database(Box::new(e))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Unauthorized(msg) => {
                tracing::warn!(reason = %msg, "Unauthorized request");
                (StatusCode::UNAUTHORIZED, Json(json!({ "error": msg }))).into_response()
            }

            AppError::Validation(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "error": msg })),
            )
                .into_response(),

            AppError::Database(e) => {
                tracing::error!("Database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "error.internal.database" })),
                )
                    .into_response()
            }

            AppError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "error.internal.unknown" })),
                )
                    .into_response()
            }
        }
    }
}
