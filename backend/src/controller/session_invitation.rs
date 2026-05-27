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
        .route("/session-invitations", get(list_handler))
        .route("/session-invitations/{id}/accept", post(accept_handler))
        .route("/session-invitations/{id}", delete(decline_handler))
        .route("/sessions/{id}/invite", post(invite_handler))
        .route("/sessions/{id}/invitable", get(invitable_handler))
}

async fn list_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    Ok(Json(
        state
            .session_invitation_svc
            .get_pending(&user.keycloak_id)
            .await?,
    ))
}

async fn accept_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state.session_invitation_svc.accept(&id, &user).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn decline_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state
        .session_invitation_svc
        .decline(&id, &user.keycloak_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn invite_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(session_id): Path<String>,
    Json(body): Json<InviteBody>,
) -> Result<StatusCode, AppError> {
    let game = state
        .session_invitation_svc
        .game_name_for_notify(&session_id)
        .await?;
    state
        .session_invitation_svc
        .invite(&session_id, &user, &body.user_id)
        .await?;

    if let Some(push) = state.push_svc.clone() {
        let invitee_id = body.user_id.clone();
        let inviter_username = user.username.clone();
        tokio::spawn(async move {
            if let Err(e) = push
                .notify_user(
                    &invitee_id,
                    "WhatsUp – Session-Einladung",
                    &format!("{inviter_username} lädt dich ein, {game} zu spielen"),
                    "/me",
                )
                .await
            {
                tracing::warn!(invitee_id = %invitee_id, error = %e, "Push notification failed (session invite)");
            }
        });
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn invitable_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(session_id): Path<String>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    Ok(Json(
        state
            .session_invitation_svc
            .get_invitable(&session_id, &user)
            .await?,
    ))
}
