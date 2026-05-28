use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Serialize;

use crate::{
    config::app_state::AppState,
    error::{AppError, codes},
    middleware::auth::AuthUser,
    model::friend_request::FriendRequest,
    repository::{
        friend_request::FriendRequestRepository,
        traits::{FriendRequestRepo, UserRepo},
        user::UserRepository,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/friends/requests", get(list_handler))
        .route(
            "/friends/requests/{id}",
            post(send_handler).delete(decline_handler),
        )
        .route("/friends/requests/{id}/accept", post(accept_handler))
}

#[derive(Serialize)]
struct FriendRequestResponse {
    id: String,
    from_id: String,
    from_username: String,
}

impl From<FriendRequest> for FriendRequestResponse {
    fn from(r: FriendRequest) -> Self {
        Self {
            id: r.id,
            from_id: r.from_id,
            from_username: r.from_username,
        }
    }
}

/// List incoming pending friend requests for the current user.
async fn list_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    let requests = FriendRequestRepository::new(Arc::clone(&state.db))
        .find_incoming(user.keycloak_id)
        .await?
        .into_iter()
        .map(FriendRequestResponse::from)
        .collect::<Vec<_>>();
    Ok(Json(requests))
}

/// Send a friend request to user {id}.
async fn send_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(to_id): Path<String>,
) -> Result<StatusCode, AppError> {
    if user.keycloak_id == to_id {
        return Err(AppError::Validation(
            codes::friend_request::SELF_REQUEST.into(),
        ));
    }

    let req_repo = FriendRequestRepository::new(Arc::clone(&state.db));
    let user_repo = UserRepository::new(Arc::clone(&state.db));

    // Already friends?
    let friend_ids = user_repo.get_friend_ids(user.keycloak_id.clone()).await?;
    if friend_ids.contains(&to_id) {
        return Err(AppError::Validation(
            codes::friend_request::ALREADY_FRIENDS.into(),
        ));
    }

    // Request already pending in either direction?
    if req_repo
        .exists_between(user.keycloak_id.clone(), to_id.clone())
        .await?
    {
        return Err(AppError::Validation(
            codes::friend_request::ALREADY_PENDING.into(),
        ));
    }

    let from_username = user.username.clone();
    req_repo
        .create(user.keycloak_id, user.username, to_id.clone())
        .await?;

    if let Some(push) = state.push_svc.clone() {
        tokio::spawn(async move {
            if let Err(e) = push
                .notify_user(
                    &to_id,
                    "WhatsUp – Freundschaftsanfrage",
                    &format!("{from_username} möchte dein Freund sein"),
                    "/me",
                )
                .await
            {
                tracing::warn!(to_id = %to_id, error = %e, "Push notification failed (friend request)");
            }
        });
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Accept an incoming friend request — adds both users as friends and deletes the request.
async fn accept_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(req_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let req_repo = FriendRequestRepository::new(Arc::clone(&state.db));
    let user_repo = UserRepository::new(Arc::clone(&state.db));

    let request = req_repo
        .find_by_id_for_receiver(req_id.clone(), user.keycloak_id.clone())
        .await?
        .ok_or_else(|| AppError::Validation(codes::friend_request::NOT_FOUND.into()))?;

    // Add both as friends (mutual)
    tokio::try_join!(
        user_repo.add_friend(request.from_id.clone(), user.keycloak_id.clone()),
        user_repo.add_friend(user.keycloak_id, request.from_id),
    )?;

    req_repo.delete(req_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Decline an incoming request or cancel an outgoing one.
async fn decline_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(req_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let deleted = FriendRequestRepository::new(Arc::clone(&state.db))
        .delete_for_participant(req_id, user.keycloak_id)
        .await?;
    if !deleted {
        return Err(AppError::Validation(
            codes::friend_request::NOT_FOUND.into(),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}
