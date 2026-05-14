mod config;
mod controller;
mod dto;
mod error;
mod middleware;
mod model;
mod repository;
mod service;

use std::sync::Arc;

use axum::{Router, extract::DefaultBodyLimit, http::{HeaderValue, Method, header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}}};
use axum::middleware::from_fn;
use tokio::sync::{RwLock, broadcast};
use tower_http::cors::CorsLayer;

use config::app_state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=info,tower_http=warn".into()),
        )
        .init();

    let settings = config::settings::Settings::load()
        .expect("Failed to load configuration — check config/app.yaml");

    tracing::info!("Configuration loaded");

    if settings.vapid.private_key.is_empty() {
        tracing::warn!("vapid.private_key not set — Web Push notifications disabled");
    }

    let db = config::db::connect(&settings.database).await?;
    let (events_tx, _) = broadcast::channel(256);

    let state = AppState {
        db,
        jwks_uri: settings.keycloak.jwks_uri.clone(),
        jwks_cache: Arc::new(RwLock::new(None)),
        events: events_tx,
        vapid_public_key: settings.vapid.public_key.clone(),
        vapid_private_key: settings.vapid.private_key.clone(),
        vapid_subject: settings.vapid.subject.clone(),
        rawg_api_key: settings.rawg.api_key.clone(),
        keycloak_admin_url: settings.keycloak.admin_url.clone(),
        keycloak_realm: settings.keycloak.realm.clone(),
        keycloak_admin_user: settings.keycloak.admin_user.clone(),
        keycloak_admin_password: settings.keycloak.admin_password.clone(),
    };

    let cors = CorsLayer::new()
        .allow_origin(settings.cors.frontend_url.parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT]);
    // expose_headers intentionally omitted — no custom headers need to be
    // readable by JS.

    let protected = controller::protected_routes().layer(axum::middleware::from_fn_with_state(
        state.clone(),
        middleware::auth::auth_middleware,
    ));

    let app = Router::new()
        .nest(
            "/api/v1",
            controller::public_routes()
                .merge(protected)
                .merge(controller::sse_routes()),
        )
        .with_state(state)
        .layer(from_fn(middleware::security::security_headers))
        .layer(cors)
        // 2 MB global JSON body limit; thumbnail upload route sets its own
        // higher limit via DefaultBodyLimit::disable() + manual size check.
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024));

    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", settings.server.port).parse()?;
    tracing::info!("Server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
