use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::{
    config::app_state::AppState,
    dto::group::GroupSummary,
    error::AppError,
    middleware::auth::AuthUser,
    repository::{group::GroupRepository, traits::GroupRepo},
    service::user::UserService,
};

#[derive(Deserialize)]
pub struct SearchParams {
    q: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users/search", get(search_handler))
        .route("/users/{id}", get(profile_handler))
        .route("/users/{id}/groups", get(public_groups_handler))
}

async fn search_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<SearchParams>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    let q = params.q.as_deref().unwrap_or("");
    let users = UserService::new(Arc::clone(&state.db))
        .search(q, &user.keycloak_id)
        .await?;
    Ok(Json(users))
}

async fn public_groups_handler(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Vec<GroupSummary>>, AppError> {
    let groups = GroupRepository::new(Arc::clone(&state.db))
        .find_public_by_member(id)
        .await?
        .into_iter()
        .map(GroupSummary::from)
        .collect();
    Ok(Json(groups))
}

async fn profile_handler(
    State(state): State<AppState>,
    Extension(requester): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    UserService::new(Arc::clone(&state.db))
        .get_profile(&id, &requester.keycloak_id)
        .await?
        .map(Json)
        .ok_or_else(|| AppError::Internal("Benutzer nicht gefunden".into()))
}
