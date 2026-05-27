use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{
    Algorithm, DecodingKey, Validation, decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
};
use serde::Deserialize;

use crate::{
    config::app_state::AppState,
    error::AppError,
    model::user::User,
    repository::{traits::UserRepo, user::UserRepository},
};

const JWKS_CACHE_TTL: Duration = Duration::from_secs(3600);

/// Extracted and validated identity injected into request extensions.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub keycloak_id: String,
    pub username: String,
    pub is_admin: bool,
}

#[derive(Debug, Deserialize)]
struct RealmAccess {
    #[serde(default)]
    roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct KeycloakClaims {
    sub: String,
    preferred_username: String,
    email: Option<String>,
    realm_access: Option<RealmAccess>,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_bearer_token(&request)?.to_owned();
    let claims = validate_token(&state, &token).await?;

    let user = User {
        keycloak_id: claims.sub.clone(),
        username: claims.preferred_username.clone(),
        email: claims.email.clone(),
    };

    UserRepository::new(Arc::clone(&state.db))
        .upsert(user)
        .await?;

    let is_admin = claims
        .realm_access
        .as_ref()
        .map(|ra| ra.roles.iter().any(|r| r == "admin"))
        .unwrap_or(false);

    let auth_user = AuthUser {
        keycloak_id: claims.sub,
        username: claims.preferred_username,
        is_admin,
    };

    tracing::debug!(
        keycloak_id = %auth_user.keycloak_id,
        username = %auth_user.username,
        is_admin = auth_user.is_admin,
        "Request authenticated"
    );

    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

fn extract_bearer_token(request: &Request) -> Result<&str, AppError> {
    let header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid Authorization header encoding".into()))?;

    header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Authorization header must use Bearer scheme".into()))
}

/// Validates a raw JWT string and returns the authenticated user.
/// Used by SSE endpoints that receive the token via query param.
pub(crate) async fn validate_query_token(
    state: &AppState,
    token: &str,
) -> Result<AuthUser, AppError> {
    let claims = validate_token(state, token).await?;
    let is_admin = claims
        .realm_access
        .as_ref()
        .map(|ra| ra.roles.iter().any(|r| r == "admin"))
        .unwrap_or(false);
    Ok(AuthUser {
        keycloak_id: claims.sub,
        username: claims.preferred_username,
        is_admin,
    })
}

async fn validate_token(state: &AppState, token: &str) -> Result<KeycloakClaims, AppError> {
    let header = decode_header(token)
        .map_err(|e| AppError::Unauthorized(format!("Invalid token header: {e}")))?;

    let kid = header
        .kid
        .ok_or_else(|| AppError::Unauthorized("Token missing 'kid' header".into()))?;

    let jwks = fetch_jwks(state).await?;

    let jwk = jwks
        .find(&kid)
        .ok_or_else(|| AppError::Unauthorized("No JWK matching token 'kid'".into()))?;

    let decoding_key = match &jwk.algorithm {
        AlgorithmParameters::RSA(rsa) => DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
            .map_err(|e| AppError::Unauthorized(format!("Invalid RSA JWK: {e}")))?,
        _ => return Err(AppError::Unauthorized("Unsupported JWK algorithm".into())),
    };

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_aud = false;

    let token_data = decode::<KeycloakClaims>(token, &decoding_key, &validation)
        .map_err(|e| AppError::Unauthorized(format!("Token validation failed: {e}")))?;

    Ok(token_data.claims)
}

async fn fetch_jwks(state: &AppState) -> Result<JwkSet, AppError> {
    {
        let cache = state.jwks_cache.read().await;
        if let Some((ref jwks, fetched_at)) = *cache
            && fetched_at.elapsed() < JWKS_CACHE_TTL
        {
            return Ok(jwks.clone());
        }
    }

    tracing::debug!("Fetching JWKS from {}", state.jwks_uri);

    let jwks: JwkSet = reqwest::get(&state.jwks_uri)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch JWKS: {e}")))?
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse JWKS response: {e}")))?;

    *state.jwks_cache.write().await = Some((jwks.clone(), Instant::now()));

    Ok(jwks)
}
