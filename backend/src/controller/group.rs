use std::convert::Infallible;
use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use futures_util::stream::StreamExt;
use serde::Deserialize;
use tokio_stream::wrappers::BroadcastStream;

use crate::{
    config::app_state::AppState,
    dto::group::{CreateGroupRequest, SetDiscordInviteRequest},
    error::AppError,
    middleware::auth::{AuthUser, validate_query_token},
    service::{group::GroupService, session::SessionService},
};

#[derive(Deserialize)]
pub struct SearchParams {
    q: Option<String>,
}

#[derive(Deserialize)]
struct SseParams {
    token: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/groups", get(search_handler).post(create_handler))
        .route("/groups/mine", get(mine_handler))
        .route("/groups/{id}", get(detail_handler).delete(delete_handler))
        .route("/groups/{id}/join", post(join_handler))
        .route(
            "/groups/{id}/discord-invite",
            axum::routing::put(set_discord_invite_handler),
        )
        .route("/groups/{id}/sessions", get(group_sessions_handler))
}

async fn set_discord_invite_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(body): Json<SetDiscordInviteRequest>,
) -> Result<StatusCode, AppError> {
    GroupService::new(Arc::clone(&state.db))
        .set_discord_invite(&id, body.url, &user.keycloak_id, user.is_admin)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// SSE route — no auth middleware, token validated via query param.
pub fn sse_routes() -> Router<AppState> {
    Router::new().route("/groups/{id}/events", get(sse_handler))
}

async fn search_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<SearchParams>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    let groups = GroupService::new(Arc::clone(&state.db))
        .search_public(query, user.is_admin)
        .await?;
    Ok(Json(groups))
}

async fn delete_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin {
        return Err(AppError::Unauthorized(
            "Nur Admins können Gruppen löschen".into(),
        ));
    }
    GroupService::new(Arc::clone(&state.db)).delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn mine_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    let groups = GroupService::new(Arc::clone(&state.db))
        .my_groups(&user.keycloak_id)
        .await?;
    Ok(Json(groups))
}

async fn detail_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    GroupService::new(Arc::clone(&state.db))
        .get_detail(&id, &user.keycloak_id)
        .await?
        .map(Json)
        .ok_or_else(|| AppError::Internal("Gruppe nicht gefunden".into()))
}

async fn group_sessions_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    let sessions = SessionService::new(Arc::clone(&state.db))
        .get_for_group(&id, &user.keycloak_id)
        .await?;
    Ok(Json(sessions))
}

async fn create_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<CreateGroupRequest>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    let group = GroupService::new(Arc::clone(&state.db))
        .create(body, &user.keycloak_id)
        .await?;
    Ok(Json(group))
}

async fn join_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    GroupService::new(Arc::clone(&state.db))
        .join(&id, &user.keycloak_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn sse_handler(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    Query(params): Query<SseParams>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, AppError> {
    // Validate token from query param (EventSource API cannot send Authorization headers)
    validate_query_token(&state, &params.token).await?;

    let rx = state.events.subscribe();

    let stream = BroadcastStream::new(rx).filter_map(move |msg| {
        let gid = group_id.clone();
        async move {
            let event = match msg {
                Ok(e) if e.group_id == gid => e,
                _ => return None,
            };
            let data = serde_json::to_string(&event.payload).unwrap_or_default();
            Some(Ok::<_, Infallible>(
                Event::default().event(&event.kind).data(data),
            ))
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
