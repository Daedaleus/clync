use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};

use crate::{config::app_state::AppState, error::AppError, middleware::auth::AuthUser};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/friends", get(list_handler))
        .route("/friends/{id}", axum::routing::delete(remove_handler))
}

async fn list_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    Ok(Json(state.user_svc.get_friends(&user.keycloak_id).await?))
}

async fn remove_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state.user_svc.remove_friend(&user.keycloak_id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
