use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use surrealdb::{Surreal, engine::remote::ws::Client};
use uuid::Uuid;

use crate::error::{AppError, codes};
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

    #[cfg(test)]
    pub(crate) fn with_repo(repo: Box<dyn crate::repository::traits::InviteRepo>) -> Self {
        Self {
            repo,
            client: reqwest::Client::new(),
            admin_url: String::new(),
            realm: String::new(),
            admin_user: String::new(),
            admin_password: String::new(),
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
                codes::registration::INVALID_CREDENTIALS.into(),
            ));
        }

        let invite = self
            .repo
            .find_valid(code.to_owned())
            .await?
            .ok_or_else(|| AppError::Validation(codes::invite::INVALID_OR_EXPIRED.into()))?;

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
                codes::registration::USERNAME_TAKEN.into(),
            )),
            status => Err(AppError::Internal(format!("Keycloak error: HTTP {status}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::{model::invite::InviteRecord, repository::traits::InviteRepo};

    struct PanicRepo;

    #[async_trait]
    impl InviteRepo for PanicRepo {
        async fn create(&self, _: String, _: String, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
        async fn find_valid(&self, _: String) -> Result<Option<InviteRecord>, surrealdb::Error> {
            unimplemented!()
        }
        async fn mark_used(&self, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
    }

    struct NoCodeRepo;

    #[async_trait]
    impl InviteRepo for NoCodeRepo {
        async fn create(&self, _: String, _: String, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
        async fn find_valid(&self, _: String) -> Result<Option<InviteRecord>, surrealdb::Error> {
            Ok(None)
        }
        async fn mark_used(&self, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn register_empty_username_is_rejected() {
        let err = InviteService::with_repo(Box::new(PanicRepo))
            .register("code", "", "password123")
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(ref msg) if msg.contains("invalid_credentials"))
        );
    }

    #[tokio::test]
    async fn register_whitespace_username_is_rejected() {
        let err = InviteService::with_repo(Box::new(PanicRepo))
            .register("code", "   ", "password123")
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(ref msg) if msg.contains("invalid_credentials"))
        );
    }

    #[tokio::test]
    async fn register_short_password_is_rejected() {
        let err = InviteService::with_repo(Box::new(PanicRepo))
            .register("code", "alice", "pass")
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(ref msg) if msg.contains("invalid_credentials"))
        );
    }

    #[tokio::test]
    async fn register_invalid_or_expired_code_is_rejected() {
        let err = InviteService::with_repo(Box::new(NoCodeRepo))
            .register("bad_code", "alice", "password123")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(ref msg) if msg.contains("invalid_or_expired")));
    }

    // ── validate ──────────────────────────────────────────────────────────────

    struct ValidCodeRepo;

    #[async_trait]
    impl InviteRepo for ValidCodeRepo {
        async fn create(&self, _: String, _: String, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn find_valid(&self, code: String) -> Result<Option<InviteRecord>, surrealdb::Error> {
            Ok(Some(InviteRecord {
                id: "inv:1".to_owned(),
                code,
                created_by: "creator".to_owned(),
                expires_at: "2099-01-01T00:00:00Z".to_owned(),
                used: false,
            }))
        }
        async fn mark_used(&self, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
    }

    #[tokio::test]
    async fn validate_returns_false_when_code_not_found() {
        let result = InviteService::with_repo(Box::new(NoCodeRepo))
            .validate("nonexistent")
            .await
            .unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn validate_returns_true_when_code_exists() {
        let result = InviteService::with_repo(Box::new(ValidCodeRepo))
            .validate("some_code")
            .await
            .unwrap();
        assert!(result);
    }

    // ── create_invite ─────────────────────────────────────────────────────────

    struct CapturingInviteRepo {
        captured: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    }

    #[async_trait]
    impl InviteRepo for CapturingInviteRepo {
        async fn create(&self, code: String, _: String, _: String) -> Result<(), surrealdb::Error> {
            *self.captured.lock().unwrap() = Some(code);
            Ok(())
        }
        async fn find_valid(&self, _: String) -> Result<Option<InviteRecord>, surrealdb::Error> {
            panic!()
        }
        async fn mark_used(&self, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
    }

    #[tokio::test]
    async fn create_invite_returns_32_char_hex_code() {
        let captured = std::sync::Arc::new(std::sync::Mutex::new(None));
        let svc = InviteService::with_repo(Box::new(CapturingInviteRepo {
            captured: captured.clone(),
        }));
        let code = svc.create_invite("creator").await.unwrap();
        assert_eq!(code.len(), 32, "UUID without dashes is 32 hex chars");
        assert!(code.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(captured.lock().unwrap().as_deref(), Some(code.as_str()));
    }
}
