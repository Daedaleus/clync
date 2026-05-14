use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};

use crate::{
    config::app_state::AppState,
    dto::push::{SubscribeRequest, VapidPublicKeyResponse},
    error::AppError,
    middleware::auth::AuthUser,
    model::push_subscription::PushSubscription,
    repository::{push_subscription::PushSubscriptionRepository, traits::PushSubscriptionRepo},
};
use std::sync::Arc;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/push/subscribe", post(subscribe_handler).delete(unsubscribe_handler))
}

/// Public endpoint — frontend needs the public key before subscribing.
pub fn public_routes() -> Router<AppState> {
    Router::new().route("/push/vapid-public-key", get(public_key_handler))
}

async fn public_key_handler(State(state): State<AppState>) -> Json<VapidPublicKeyResponse> {
    Json(VapidPublicKeyResponse { public_key: state.vapid_public_key.clone() })
}

async fn subscribe_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<SubscribeRequest>,
) -> Result<StatusCode, AppError> {
    PushSubscriptionRepository::new(Arc::clone(&state.db))
        .upsert(PushSubscription {
            user_id: user.keycloak_id,
            endpoint: body.endpoint,
            p256dh: body.p256dh,
            auth: body.auth,
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn unsubscribe_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<StatusCode, AppError> {
    PushSubscriptionRepository::new(Arc::clone(&state.db))
        .delete(user.keycloak_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
