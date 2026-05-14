use std::sync::Arc;
use std::time::Instant;

use jsonwebtoken::jwk::JwkSet;
use serde::Serialize;
use serde_json::Value;
use surrealdb::{Surreal, engine::remote::ws::Client};
use tokio::sync::{RwLock, broadcast};

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
    /// VAPID private key (base64url) — used server-side to sign push messages.
    pub vapid_private_key: String,
    /// VAPID subject — mailto: or https: URI identifying the sender.
    pub vapid_subject: String,
    /// RAWG API key — empty string disables the autofill feature.
    pub rawg_api_key: String,
    pub keycloak_admin_url: String,
    pub keycloak_realm: String,
    pub keycloak_admin_user: String,
    pub keycloak_admin_password: String,
}
