use axum::{Json, Router, routing::get};

use crate::config::app_state::AppState;
use crate::dto::health::HealthResponse;
use crate::service::health::get_health;

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health_handler))
}

async fn health_handler() -> Json<HealthResponse> {
    Json(get_health())
}
