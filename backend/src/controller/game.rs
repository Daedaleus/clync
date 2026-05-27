use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use serde::Deserialize;

use crate::{
    config::app_state::AppState,
    dto::game::{AddGameRequest, RemoveGameQuery},
    error::AppError,
    middleware::auth::AuthUser,
};

#[derive(Deserialize)]
pub struct SearchParams {
    q: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/games", get(suggestions_handler))
        .route("/me/games", post(add_handler))
        .route("/me/games", delete(remove_handler))
}

async fn suggestions_handler(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<String>>, AppError> {
    let q = params.q.as_deref().unwrap_or("");
    Ok(Json(state.game_svc.suggestions(q).await?))
}

async fn add_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<AddGameRequest>,
) -> Result<StatusCode, AppError> {
    state.game_svc.add(&user.keycloak_id, &body.name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<RemoveGameQuery>,
) -> Result<StatusCode, AppError> {
    state
        .game_svc
        .remove(&user.keycloak_id, &params.name)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
