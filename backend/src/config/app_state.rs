use std::sync::Arc;
use std::time::Instant;

use jsonwebtoken::jwk::JwkSet;
use serde::Serialize;
use serde_json::Value;
use surrealdb::{Surreal, engine::remote::ws::Client};
use tokio::sync::{RwLock, broadcast};

use crate::service::{
    game::GameService, group::GroupService, invitation::InvitationService, invite::InviteService,
    me::MeService, push::PushService, session::SessionService,
    session_invitation::SessionInvitationService, user::UserService,
};

/// Event broadcast to all SSE subscribers of a specific group.
#[derive(Clone, Debug, Serialize)]
pub struct GroupEvent {
    pub group_id: String,
    /// "session_created" | "session_joined" | "session_deleted"
    pub kind: String,
    pub payload: Value,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Surreal<Client>>,
    pub jwks_uri: String,
    /// Cached JWKS with fetch timestamp for TTL checks.
    pub jwks_cache: Arc<RwLock<Option<(JwkSet, Instant)>>>,
    /// Broadcast channel for real-time group events (SSE).
    pub events: broadcast::Sender<GroupEvent>,
    /// VAPID public key (base64url) — sent to frontend for push subscription.
    pub vapid_public_key: String,
    /// RAWG API key — empty string disables the autofill feature.
    pub rawg_api_key: String,
    // ── Services (created once at startup, shared across requests) ────────────
    pub session_svc: Arc<SessionService>,
    pub group_svc: Arc<GroupService>,
    pub game_svc: Arc<GameService>,
    pub user_svc: Arc<UserService>,
    pub me_svc: Arc<MeService>,
    pub invitation_svc: Arc<InvitationService>,
    pub session_invitation_svc: Arc<SessionInvitationService>,
    pub invite_svc: Arc<InviteService>,
    /// None when VAPID key is not configured (push notifications disabled).
    pub push_svc: Option<Arc<PushService>>,
}
