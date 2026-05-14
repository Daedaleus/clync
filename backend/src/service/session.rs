use std::sync::Arc;

use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::dto::session::{CreateSessionRequest, SessionDetailResponse, SessionResponse};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::repository::{
    group::GroupRepository,
    session::SessionRepository,
    traits::{GroupRepo, SessionRepo},
};

pub struct SessionService {
    repo: Box<dyn SessionRepo>,
    group_repo: Box<dyn GroupRepo>,
}

impl SessionService {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self {
            repo: Box::new(SessionRepository::new(Arc::clone(&db))),
            group_repo: Box::new(GroupRepository::new(db)),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_repos(repo: Box<dyn SessionRepo>, group_repo: Box<dyn GroupRepo>) -> Self {
        Self { repo, group_repo }
    }

    pub async fn create(
        &self,
        user: &AuthUser,
        req: CreateSessionRequest,
    ) -> Result<SessionResponse, AppError> {
        validate_create_request(&req)?;
        let game = req.game.trim().to_owned();
        let session = self
            .repo
            .create(
                user.keycloak_id.clone(),
                user.username.clone(),
                game,
                req.scheduled_at,
                req.scope,
                req.group_ids,
            )
            .await?
            .ok_or_else(|| AppError::Internal("Session creation returned no result".into()))?;
        Ok(session.into_response(&user.keycloak_id))
    }

    pub async fn get_feed(&self, user: &AuthUser) -> Result<Vec<SessionResponse>, AppError> {
        let my_group_ids = self
            .group_repo
            .find_by_member(user.keycloak_id.clone())
            .await?
            .into_iter()
            .map(|g| g.id)
            .collect();
        let sessions = self.repo.find_feed(my_group_ids).await?;
        Ok(sessions
            .into_iter()
            .map(|s| s.into_response(&user.keycloak_id))
            .collect())
    }

    pub async fn get_mine(&self, user: &AuthUser) -> Result<Vec<SessionResponse>, AppError> {
        let sessions = self.repo.find_mine(user.keycloak_id.clone()).await?;
        Ok(sessions
            .into_iter()
            .map(|s| s.into_response(&user.keycloak_id))
            .collect())
    }

    pub async fn get_for_group(
        &self,
        group_id: &str,
        requester_id: &str,
    ) -> Result<Vec<SessionResponse>, AppError> {
        let sessions = self.repo.find_for_group(group_id.to_owned()).await?;
        Ok(sessions
            .into_iter()
            .map(|s| s.into_response(requester_id))
            .collect())
    }

    pub async fn get_detail(
        &self,
        session_id: &str,
        requester_id: &str,
    ) -> Result<Option<SessionDetailResponse>, AppError> {
        Ok(self
            .repo
            .find_by_id(session_id.to_owned())
            .await?
            .map(|s| s.into_detail_response(requester_id)))
    }

    /// Returns the updated session (for broadcasting), None if caller is the creator.
    pub async fn join(
        &self,
        session_id: &str,
        user: &AuthUser,
    ) -> Result<Option<SessionResponse>, AppError> {
        let session = self
            .repo
            .join(session_id.to_owned(), user.keycloak_id.clone())
            .await?;
        Ok(session.map(|s| s.into_response(&user.keycloak_id)))
    }

    pub async fn leave(&self, session_id: &str, user_id: &str) -> Result<(), AppError> {
        self.repo
            .leave(session_id.to_owned(), user_id.to_owned())
            .await?;
        Ok(())
    }

    /// Returns the group_ids for broadcast routing.
    pub async fn delete(
        &self,
        session_id: &str,
        user_id: &str,
        is_admin: bool,
    ) -> Result<Vec<String>, AppError> {
        if is_admin {
            Ok(self.repo.delete_as_admin(session_id.to_owned()).await?)
        } else {
            Ok(self
                .repo
                .delete(session_id.to_owned(), user_id.to_owned())
                .await?)
        }
    }
}

pub(crate) fn validate_create_request(req: &CreateSessionRequest) -> Result<(), AppError> {
    if req.game.trim().is_empty() {
        return Err(AppError::Validation(
            "Spielname darf nicht leer sein".into(),
        ));
    }
    if req.scheduled_at.is_empty() {
        return Err(AppError::Validation("Datum darf nicht leer sein".into()));
    }
    let scheduled = chrono::DateTime::parse_from_rfc3339(&req.scheduled_at)
        .map_err(|_| AppError::Validation("Ungültiges Datumsformat".into()))?;
    if scheduled <= chrono::Utc::now() {
        return Err(AppError::Validation(
            "Session kann nicht in der Vergangenheit liegen".into(),
        ));
    }
    if req.scope != "global" && req.scope != "groups" {
        return Err(AppError::Validation(
            "Ungültiger Scope (global oder groups)".into(),
        ));
    }
    if req.scope == "groups" && req.group_ids.is_empty() {
        return Err(AppError::Validation(
            "Mindestens eine Gruppe auswählen".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(
        game: &str,
        scheduled_at: &str,
        scope: &str,
        group_ids: Vec<&str>,
    ) -> CreateSessionRequest {
        CreateSessionRequest {
            game: game.into(),
            scheduled_at: scheduled_at.into(),
            scope: scope.into(),
            group_ids: group_ids.into_iter().map(String::from).collect(),
        }
    }

    // ── validate_create_request ────────────────────────────────────────────────

    #[test]
    fn empty_game_name_is_rejected() {
        assert!(
            validate_create_request(&req("", "2026-05-20T18:00:00Z", "global", vec![])).is_err()
        );
    }

    #[test]
    fn whitespace_only_game_name_is_rejected() {
        assert!(
            validate_create_request(&req("   ", "2026-05-20T18:00:00Z", "global", vec![])).is_err()
        );
    }

    #[test]
    fn empty_datetime_is_rejected() {
        assert!(validate_create_request(&req("CS2", "", "global", vec![])).is_err());
    }

    #[test]
    fn invalid_scope_is_rejected() {
        assert!(
            validate_create_request(&req("CS2", "2026-05-20T18:00:00Z", "public", vec![])).is_err()
        );
    }

    #[test]
    fn groups_scope_without_group_ids_is_rejected() {
        assert!(
            validate_create_request(&req("CS2", "2026-05-20T18:00:00Z", "groups", vec![])).is_err()
        );
    }

    #[test]
    fn global_scope_is_valid() {
        assert!(
            validate_create_request(&req("CS2", "2026-05-20T18:00:00Z", "global", vec![])).is_ok()
        );
    }

    #[test]
    fn groups_scope_with_group_id_is_valid() {
        assert!(
            validate_create_request(&req(
                "CS2",
                "2026-05-20T18:00:00Z",
                "groups",
                vec!["group1"]
            ))
            .is_ok()
        );
    }
}
