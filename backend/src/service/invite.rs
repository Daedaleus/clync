use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use surrealdb::{Surreal, engine::remote::ws::Client};
use uuid::Uuid;

use crate::error::AppError;
use crate::repository::{invite::InviteRepository, traits::InviteRepo};

pub struct InviteService {
    repo: Box<dyn InviteRepo>,
    client: reqwest::Client,
    admin_url: String,
    realm: String,
    admin_user: String,
    admin_password: String,
}

impl InviteService {
    pub fn new(
        db: Arc<Surreal<Client>>,
        admin_url: String,
        realm: String,
        admin_user: String,
        admin_password: String,
    ) -> Self {
        Self {
            repo: Box::new(InviteRepository::new(db)),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("reqwest client construction failed"),
            admin_url,
            realm,
            admin_user,
            admin_password,
        }
    }

    /// Creates a single-use invite token valid for 7 days.
    pub async fn create_invite(&self, creator_id: &str) -> Result<String, AppError> {
        let code = Uuid::new_v4().to_string().replace('-', "");
        let expires_at = (chrono::Utc::now() + chrono::Duration::days(7)).to_rfc3339();
        self.repo
            .create(code.clone(), creator_id.to_owned(), expires_at)
            .await?;
        tracing::info!(creator_id = %creator_id, "Invite created");
        Ok(code)
    }

    pub async fn validate(&self, code: &str) -> Result<bool, AppError> {
        Ok(self.repo.find_valid(code.to_owned()).await?.is_some())
    }

    pub async fn register(
        &self,
        code: &str,
        username: &str,
        password: &str,
    ) -> Result<(), AppError> {
        if username.trim().is_empty() || password.len() < 6 {
            return Err(AppError::Validation(
                "error.registration.invalid_credentials".into(),
            ));
        }

        let invite = self
            .repo
            .find_valid(code.to_owned())
            .await?
            .ok_or_else(|| AppError::Validation("error.invite.invalid_or_expired".into()))?;

        let admin_token = self.get_admin_token().await?;
        self.create_keycloak_user(&admin_token, username, password)
            .await?;
        self.repo.mark_used(invite.code).await?;
        tracing::info!(username = %username, "New user registered via invite");
        Ok(())
    }

    async fn get_admin_token(&self) -> Result<String, AppError> {
        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
        }

        let resp = self
            .client
            .post(format!(
                "{}/realms/master/protocol/openid-connect/token",
                self.admin_url
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", &self.admin_user),
                ("password", &self.admin_password),
            ])
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Keycloak admin auth failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::Internal(
                "Keycloak admin authentication failed".into(),
            ));
        }

        let body: TokenResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Keycloak token parse error: {e}")))?;
        Ok(body.access_token)
    }

    async fn create_keycloak_user(
        &self,
        admin_token: &str,
        username: &str,
        password: &str,
    ) -> Result<(), AppError> {
        #[derive(Serialize)]
        struct Credential {
            r#type: &'static str,
            value: String,
            temporary: bool,
        }
        #[derive(Serialize)]
        struct NewUser {
            username: String,
            enabled: bool,
            credentials: Vec<Credential>,
        }

        let resp = self
            .client
            .post(format!(
                "{}/admin/realms/{}/users",
                self.admin_url, self.realm
            ))
            .bearer_auth(admin_token)
            .json(&NewUser {
                username: username.to_owned(),
                enabled: true,
                credentials: vec![Credential {
                    r#type: "password",
                    value: password.to_owned(),
                    temporary: false,
                }],
            })
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Keycloak create user failed: {e}")))?;

        match resp.status().as_u16() {
            201 => Ok(()),
            409 => Err(AppError::Validation(
                "error.registration.username_taken".into(),
            )),
            status => Err(AppError::Internal(format!("Keycloak error: HTTP {status}"))),
        }
    }
}
