mod config;
mod controller;
mod dto;
mod error;
mod middleware;
mod model;
mod repository;
mod service;

use std::sync::Arc;

use axum::middleware::from_fn;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{
        HeaderValue, Method, Request,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
};
use tokio::sync::{RwLock, broadcast};
use tokio::time::Duration;
use tower_http::{
    LatencyUnit,
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnFailure, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use config::app_state::AppState;
use service::{
    game::GameService, group::GroupService, invitation::InvitationService, invite::InviteService,
    me::MeService, push::PushService, session::SessionService,
    session_invitation::SessionInvitationService, user::UserService,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .json()
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
    config::migrations::run(&db).await?;
    let (events_tx, _) = broadcast::channel(256);

    let push_svc = match PushService::new(
        Arc::clone(&db),
        settings.vapid.private_key.clone(),
        settings.vapid.subject.clone(),
    ) {
        Ok(svc) => {
            if settings.vapid.private_key.is_empty() {
                None
            } else {
                Some(Arc::new(svc))
            }
        }
        Err(e) => {
            tracing::error!("PushService init failed: {e} — push notifications disabled");
            None
        }
    };

    let state = AppState {
        jwks_uri: settings.keycloak.jwks_uri.clone(),
        jwks_cache: Arc::new(RwLock::new(None)),
        events: events_tx,
        vapid_public_key: settings.vapid.public_key.clone(),
        rawg_api_key: settings.rawg.api_key.clone(),
        session_svc: Arc::new(SessionService::new(Arc::clone(&db))),
        group_svc: Arc::new(GroupService::new(Arc::clone(&db))),
        game_svc: Arc::new(GameService::new(Arc::clone(&db))),
        user_svc: Arc::new(UserService::new(Arc::clone(&db))),
        me_svc: Arc::new(MeService::new(Arc::clone(&db))),
        invitation_svc: Arc::new(InvitationService::new(Arc::clone(&db))),
        session_invitation_svc: Arc::new(SessionInvitationService::new(Arc::clone(&db))),
        invite_svc: Arc::new(InviteService::new(
            Arc::clone(&db),
            settings.keycloak.admin_url.clone(),
            settings.keycloak.realm.clone(),
            settings.keycloak.admin_user.clone(),
            settings.keycloak.admin_password.clone(),
        )),
        push_svc,
        db,
    };

    let cors = CorsLayer::new()
        .allow_origin(settings.cors.frontend_url.parse::<HeaderValue>()?)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT])
        // Expose x-request-id so the browser JS can read it for error correlation.
        .expose_headers([axum::http::HeaderName::from_static("x-request-id")]);

    let protected = controller::protected_routes().layer(axum::middleware::from_fn_with_state(
        state.clone(),
        middleware::auth::auth_middleware,
    ));

    // Background task: send push notifications for sessions starting within 5 minutes.
    let notify_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let repo = repository::session::SessionRepository::new(Arc::clone(&notify_state.db));
            use repository::traits::SessionRepo;
            match repo.find_sessions_to_notify().await {
                Ok(sessions) if !sessions.is_empty() => {
                    for session in sessions {
                        if let Some(push) = &notify_state.push_svc {
                            let mut user_ids = vec![session.user_id.clone()];
                            user_ids.extend(session.participants.iter().cloned());
                            user_ids.dedup();
                            if let Err(e) = push
                                .notify_session_start(user_ids, &session.game, &session.id)
                                .await
                            {
                                tracing::warn!(
                                    session_id = %session.id,
                                    error = %e,
                                    "Session start push notification failed"
                                );
                            }
                        }
                        if let Err(e) = repo.mark_start_notified(session.id).await {
                            tracing::error!(error = %e, "mark_start_notified failed");
                        }
                    }
                }
                Err(e) => tracing::error!(error = %e, "find_sessions_to_notify failed"),
                _ => {}
            }
        }
    });

    // Background task: purge stale data hourly.
    // - Sessions more than 24 h past their scheduled_at
    // - Session invitations whose session has already started (2 h grace)
    let cleanup_db = Arc::clone(&state.db);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let queries = [
                "DELETE session WHERE scheduled_at < time::now() - 24h",
                "DELETE session_invitation WHERE scheduled_at < time::now() - 2h",
            ];
            for q in &queries {
                if let Err(e) = cleanup_db.query(*q).await {
                    tracing::error!(query = q, error = %e, "Cleanup query failed");
                }
            }
            tracing::debug!("Cleanup: stale sessions and session invitations purged");
        }
    });

    let app = Router::new()
        .nest(
            "/api/v1",
            controller::public_routes()
                .merge(protected)
                .merge(controller::sse_routes()),
        )
        .with_state(state)
        // Innermost: add security headers
        .layer(from_fn(middleware::security::security_headers))
        // HTTP access logging — span carries request_id + method + path;
        // all log calls within a handler automatically inherit these fields.
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request<_>| {
                    let request_id = req
                        .headers()
                        .get("x-request-id")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("-");
                    tracing::info_span!(
                        "request",
                        request_id = request_id,
                        method = %req.method(),
                        path = %req.uri().path(),
                    )
                })
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::DEBUG)
                        .latency_unit(LatencyUnit::Millis),
                )
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        )
        // Propagate x-request-id to response headers
        .layer(PropagateRequestIdLayer::x_request_id())
        // Generate a UUID x-request-id for every incoming request
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(cors)
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024));

    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", settings.server.port).parse()?;
    tracing::info!(addr = %addr, "Server listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
