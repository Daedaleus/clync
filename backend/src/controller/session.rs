use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get},
};
use serde_json::json;

use crate::{
    config::app_state::{AppState, GroupEvent},
    dto::session::CreateSessionRequest,
    error::AppError,
    middleware::auth::AuthUser,
    service::{push::PushService, session::SessionService},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/sessions", get(feed_handler).post(create_handler))
        .route("/sessions/mine", get(mine_handler))
        .route("/sessions/{id}", get(detail_handler).delete(delete_handler))
        .route("/sessions/{id}/join", axum::routing::post(join_handler).delete(leave_handler))
}

async fn detail_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    SessionService::new(Arc::clone(&state.db))
        .get_detail(&id, &user.keycloak_id)
        .await?
        .map(Json)
        .ok_or_else(|| AppError::Internal("Session nicht gefunden".into()))
}

async fn mine_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    let sessions = SessionService::new(Arc::clone(&state.db)).get_mine(&user).await?;
    Ok(Json(sessions))
}

async fn feed_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    let sessions = SessionService::new(Arc::clone(&state.db)).get_feed(&user).await?;
    Ok(Json(sessions))
}

async fn create_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<CreateSessionRequest>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    let group_ids = body.group_ids.clone();
    let session = SessionService::new(Arc::clone(&state.db)).create(&user, body).await?;

    for group_id in &group_ids {
        let _ = state.events.send(GroupEvent {
            group_id: group_id.clone(),
            kind: "session_created".into(),
            payload: serde_json::to_value(&session).unwrap_or_default(),
        });
    }

    // Send Web Push to group members in the background (non-blocking)
    if !group_ids.is_empty() {
        let session_clone = session.clone();
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Ok(push_svc) = PushService::new(
                Arc::clone(&state_clone.db),
                state_clone.vapid_private_key.clone(),
                state_clone.vapid_subject.clone(),
            ) {
                let _ = push_svc.notify_groups(&group_ids, &session_clone).await;
            }
        });
    }

    Ok(Json(session))
}

async fn join_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if let Some(session) = SessionService::new(Arc::clone(&state.db)).join(&id, &user).await? {
        for group_id in &session.group_ids {
            let _ = state.events.send(GroupEvent {
                group_id: group_id.clone(),
                kind: "session_joined".into(),
                payload: json!({ "id": session.id, "participant_count": session.participant_count }),
            });
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn leave_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    SessionService::new(Arc::clone(&state.db)).leave(&id, &user.keycloak_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let group_ids = SessionService::new(Arc::clone(&state.db))
        .delete(&id, &user.keycloak_id, user.is_admin)
        .await?;

    for group_id in group_ids {
        let _ = state.events.send(GroupEvent {
            group_id,
            kind: "session_deleted".into(),
            payload: json!({ "id": id }),
        });
    }

    Ok(StatusCode::NO_CONTENT)
}
