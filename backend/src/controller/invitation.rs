use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use serde::Deserialize;

use crate::config::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::service::invitation::InvitationService;

#[derive(Deserialize)]
struct InviteBody {
    user_id: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        // Pending invitations for the current user
        .route("/invitations", get(list_handler))
        .route("/invitations/{id}/accept", post(accept_handler))
        .route("/invitations/{id}", delete(decline_handler))
        // Group-scoped invitation actions
        .route("/groups/{id}/invite", post(invite_handler))
        .route("/groups/{id}/invitable", get(invitable_handler))
}

async fn list_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    let invitations = InvitationService::new(Arc::clone(&state.db))
        .get_pending(&user.keycloak_id)
        .await?;
    Ok(Json(invitations))
}

async fn accept_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    InvitationService::new(Arc::clone(&state.db))
        .accept(&id, &user)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn decline_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    InvitationService::new(Arc::clone(&state.db))
        .decline(&id, &user.keycloak_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn invite_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(group_id): Path<String>,
    Json(body): Json<InviteBody>,
) -> Result<StatusCode, AppError> {
    InvitationService::new(Arc::clone(&state.db))
        .invite(&group_id, &user, &body.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn invitable_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(group_id): Path<String>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    let users = InvitationService::new(Arc::clone(&state.db))
        .get_invitable(&group_id, &user)
        .await?;
    Ok(Json(users))
}
