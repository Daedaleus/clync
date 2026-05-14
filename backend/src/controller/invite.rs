use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::config::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::service::invite::InviteService;

fn svc(state: &AppState) -> InviteService {
    InviteService::new(
        Arc::clone(&state.db),
        state.keycloak_admin_url.clone(),
        state.keycloak_realm.clone(),
        state.keycloak_admin_user.clone(),
        state.keycloak_admin_password.clone(),
    )
}

pub fn protected_routes() -> Router<AppState> {
    Router::new().route("/invites", post(create_handler))
}

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/invites/{token}", get(validate_handler))
        .route("/invites/{token}/register", post(register_handler))
}

#[derive(Serialize)]
struct CreateResponse {
    token: String,
} // keep field name "token" for frontend compat

async fn create_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<CreateResponse>, AppError> {
    let code = svc(&state).create_invite(&user.keycloak_id).await?;
    Ok(Json(CreateResponse { token: code }))
}

#[derive(Serialize)]
struct ValidateResponse {
    valid: bool,
}

async fn validate_handler(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<ValidateResponse>, AppError> {
    let valid = svc(&state).validate(&code).await?;
    Ok(Json(ValidateResponse { valid }))
}

#[derive(Deserialize)]
struct RegisterBody {
    username: String,
    password: String,
}

async fn register_handler(
    State(state): State<AppState>,
    Path(code): Path<String>,
    Json(body): Json<RegisterBody>,
) -> Result<StatusCode, AppError> {
    svc(&state)
        .register(&code, &body.username, &body.password)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
