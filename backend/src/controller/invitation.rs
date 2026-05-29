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
    Ok(Json(
        state.invitation_svc.get_pending(&user.keycloak_id).await?,
    ))
}

async fn accept_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state.invitation_svc.accept(&id, &user).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn decline_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state.invitation_svc.decline(&id, &user.keycloak_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn invite_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(group_id): Path<String>,
    Json(body): Json<InviteBody>,
) -> Result<StatusCode, AppError> {
    let group_name = state
        .invitation_svc
        .group_name_for_notify(&group_id)
        .await?;
    state
        .invitation_svc
        .invite(&group_id, &user, &body.user_id)
        .await?;

    if let Some(push) = state.push_svc.clone() {
        let invitee_id = body.user_id.clone();
        let inviter_username = user.username.clone();
        tokio::spawn(async move {
            if let Err(e) = push
                .notify_user(
                    &invitee_id,
                    "Clync – Gruppeneinladung",
                    &format!("{inviter_username} hat dich zu \"{group_name}\" eingeladen"),
                    "/me",
                )
                .await
            {
                tracing::warn!(invitee_id = %invitee_id, error = %e, "Push notification failed (group invite)");
            }
        });
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn invitable_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(group_id): Path<String>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    Ok(Json(
        state.invitation_svc.get_invitable(&group_id, &user).await?,
    ))
}
