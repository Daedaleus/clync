use axum::{
    Extension, Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};

use crate::config::app_state::AppState;
use crate::dto::game::{UpdateGameRequest, UpsertGameRequest};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/library", get(list_handler).post(create_handler))
        .route(
            "/library/{name}",
            get(detail_handler)
                .put(update_handler)
                .delete(delete_handler),
        )
        .route(
            "/library/{name}/thumbnail",
            post(upload_thumbnail_handler).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/library/{name}/autofill",
            get(autofill_candidates_handler).post(autofill_confirm_handler),
        )
}

/// Public route — no auth required so <img> tags can load thumbnails directly.
pub fn public_thumbnail_route() -> Router<AppState> {
    Router::new().route("/library/{name}/thumbnail", get(serve_thumbnail_handler))
}

async fn list_handler(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    Ok(Json(state.game_svc.list_all().await?))
}

async fn detail_handler(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
    Path(name): Path<String>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    state
        .game_svc
        .get_by_name(&name)
        .await?
        .map(Json)
        .ok_or_else(|| AppError::Internal("Spiel nicht gefunden".into()))
}

async fn create_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<UpsertGameRequest>,
) -> Result<StatusCode, AppError> {
    state.game_svc.create(body, &user.keycloak_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_handler(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
    Path(name): Path<String>,
    Json(body): Json<UpdateGameRequest>,
) -> Result<StatusCode, AppError> {
    state.game_svc.update(&name, body).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn upload_thumbnail_handler(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
    Path(name): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        if field.name() != Some("file") {
            continue;
        }

        let content_type = field.content_type().unwrap_or("image/jpeg").to_owned();

        let bytes = field
            .bytes()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .to_vec();

        const MAX_BYTES: usize = 5 * 1024 * 1024; // 5 MB
        if bytes.len() > MAX_BYTES {
            return Err(AppError::Validation(
                "error.game.thumbnail_too_large".into(),
            ));
        }

        state
            .game_svc
            .store_thumbnail(&name, bytes, content_type)
            .await?;

        let url = format!("/api/v1/library/{}/thumbnail", name);
        return Ok(Json(serde_json::json!({ "url": url })));
    }

    Err(AppError::Internal("Kein Datei-Feld gefunden".into()))
}

async fn delete_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    state.game_svc.delete_game(&name, user.is_admin).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn autofill_candidates_handler(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
    Path(name): Path<String>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    let candidates = state
        .game_svc
        .get_autofill_candidates(&name, &state.rawg_api_key)
        .await?;
    Ok(Json(candidates))
}

#[derive(serde::Deserialize)]
struct AutofillConfirmBody {
    rawg_id: u64,
    genre: Option<String>,
}

async fn autofill_confirm_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(name): Path<String>,
    Json(body): Json<AutofillConfirmBody>,
) -> Result<Json<impl serde::Serialize>, AppError> {
    let game = state
        .game_svc
        .confirm_autofill(
            &name,
            body.rawg_id,
            &state.rawg_api_key,
            body.genre,
            &user.keycloak_id,
        )
        .await?;
    Ok(Json(game))
}

async fn serve_thumbnail_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.game_svc.get_thumbnail(&name).await {
        Ok(Some((bytes, content_type))) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, content_type),
                (header::CACHE_CONTROL, "public, max-age=86400".to_owned()),
            ],
            bytes,
        )
            .into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}
