use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, put},
};

use crate::config::app_state::AppState;
use crate::dto::me::UpdateProfileRequest;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(me_handler))
        .route("/me/profile", put(update_profile_handler))
}

async fn me_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    Ok(Json(state.me_svc.get_me(&user).await?))
}

async fn update_profile_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<StatusCode, AppError> {
    state.me_svc.update_profile(&user.keycloak_id, req).await?;
    Ok(StatusCode::NO_CONTENT)
}
