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
        let notes = req
            .notes
            .map(|n| n.trim().to_owned())
            .filter(|n| !n.is_empty());
        let session = self
            .repo
            .create(
                user.keycloak_id.clone(),
                user.username.clone(),
                game,
                req.scheduled_at,
                req.scope,
                req.group_ids,
                notes,
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

    /// Set RSVP status. Returns updated session (for SSE broadcast), None if caller is the creator.
    pub async fn set_rsvp(
        &self,
        session_id: &str,
        user: &AuthUser,
        status: String,
    ) -> Result<Option<SessionResponse>, AppError> {
        if status != "accepted" && status != "maybe" && status != "declined" {
            return Err(AppError::Validation("error.session.invalid_rsvp".into()));
        }
        let session = self
            .repo
            .set_rsvp(
                session_id.to_owned(),
                user.keycloak_id.clone(),
                user.username.clone(),
                status,
            )
            .await?;
        Ok(session.map(|s| s.into_response(&user.keycloak_id)))
    }

    /// Remove RSVP. Returns updated session (for SSE broadcast), None if caller is the creator.
    pub async fn remove_rsvp(
        &self,
        session_id: &str,
        user: &AuthUser,
    ) -> Result<Option<SessionResponse>, AppError> {
        let session = self
            .repo
            .remove_rsvp(session_id.to_owned(), user.keycloak_id.clone())
            .await?;
        Ok(session.map(|s| s.into_response(&user.keycloak_id)))
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
        return Err(AppError::Validation("error.game.name_required".into()));
    }
    if req.scheduled_at.is_empty() {
        return Err(AppError::Validation("error.session.date_required".into()));
    }
    let scheduled = chrono::DateTime::parse_from_rfc3339(&req.scheduled_at)
        .map_err(|_| AppError::Validation("error.session.invalid_date".into()))?;
    if scheduled <= chrono::Utc::now() {
        return Err(AppError::Validation("error.session.date_in_past".into()));
    }
    if req.scope != "global" && req.scope != "groups" {
        return Err(AppError::Validation("error.session.invalid_scope".into()));
    }
    if req.scope == "groups" && req.group_ids.is_empty() {
        return Err(AppError::Validation("error.session.groups_required".into()));
    }
    if let Some(notes) = &req.notes
        && notes.trim().len() > 500
    {
        return Err(AppError::Validation("error.session.notes_too_long".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::{
        middleware::auth::AuthUser,
        model::{
            group::Group,
            session::{Session, SessionDetail},
        },
        repository::traits::{GroupRepo, SessionRepo},
    };

    // ── Mock helpers ──────────────────────────────────────────────────────────

    struct FakeSessionRepo {
        feed: Vec<Session>,
        mine: Vec<Session>,
        for_group: Vec<Session>,
    }

    #[async_trait]
    impl SessionRepo for FakeSessionRepo {
        async fn create(
            &self,
            _: String,
            _: String,
            _: String,
            _: String,
            _: String,
            _: Vec<String>,
            _: Option<String>,
        ) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_feed(&self, _: Vec<String>) -> Result<Vec<Session>, surrealdb::Error> {
            Ok(self.feed.clone())
        }
        async fn find_mine(&self, _: String) -> Result<Vec<Session>, surrealdb::Error> {
            Ok(self.mine.clone())
        }
        async fn find_for_group(&self, _: String) -> Result<Vec<Session>, surrealdb::Error> {
            Ok(self.for_group.clone())
        }
        async fn find_by_id(&self, _: String) -> Result<Option<SessionDetail>, surrealdb::Error> {
            unimplemented!()
        }
        async fn join(&self, _: String, _: String) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn leave(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
        async fn set_rsvp(
            &self,
            _: String,
            _: String,
            _: String,
            _: String,
        ) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn remove_rsvp(
            &self,
            _: String,
            _: String,
        ) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn delete(&self, _: String, _: String) -> Result<Vec<String>, surrealdb::Error> {
            unimplemented!()
        }
        async fn delete_as_admin(&self, _: String) -> Result<Vec<String>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_sessions_to_notify(
            &self,
        ) -> Result<Vec<crate::model::session::SessionStartReminder>, surrealdb::Error> {
            unimplemented!()
        }
        async fn mark_start_notified(&self, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
    }

    struct FakeGroupRepo;

    #[async_trait]
    impl GroupRepo for FakeGroupRepo {
        async fn find_by_member(&self, _: String) -> Result<Vec<Group>, surrealdb::Error> {
            Ok(vec![])
        }
        async fn create(
            &self,
            _: String,
            _: bool,
            _: String,
        ) -> Result<Option<Group>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_by_id(&self, _: String) -> Result<Option<Group>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_member_games(
            &self,
            _: Vec<String>,
        ) -> Result<Vec<crate::model::group::MemberWithGames>, surrealdb::Error> {
            unimplemented!()
        }
        async fn list_public(&self) -> Result<Vec<Group>, surrealdb::Error> {
            unimplemented!()
        }
        async fn search_public(&self, _: String) -> Result<Vec<Group>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_public_by_member(&self, _: String) -> Result<Vec<Group>, surrealdb::Error> {
            unimplemented!()
        }
        async fn join(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
        async fn share_group(&self, _: String, _: String) -> Result<bool, surrealdb::Error> {
            unimplemented!()
        }
        async fn delete(&self, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
        async fn find_all(&self) -> Result<Vec<Group>, surrealdb::Error> {
            unimplemented!()
        }
        async fn set_discord_invite(
            &self,
            _: String,
            _: Option<String>,
        ) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
    }

    fn session_at(scheduled_at: &str, scope: &str) -> Session {
        Session {
            id: "s1".into(),
            user_id: "u1".into(),
            username: "Alice".into(),
            game: "CS2".into(),
            scheduled_at: scheduled_at.into(),
            scope: scope.into(),
            group_ids: if scope == "groups" {
                vec!["g1".into()]
            } else {
                vec![]
            },
            participants: vec![],
            group_names: vec![],
            game_has_thumbnail: false,
            notes: None,
            rsvps: vec![],
        }
    }

    fn alice() -> AuthUser {
        AuthUser {
            keycloak_id: "u1".into(),
            username: "Alice".into(),
            is_admin: false,
        }
    }

    // ── Regression: sessions with future scheduled_at must not be filtered out ─
    //
    // Before the fix, scheduled_at was stored as a SurrealDB string. Comparing
    // it to time::now() (a datetime) always yielded false, so every query
    // returned an empty list regardless of when the session was scheduled.

    #[tokio::test]
    async fn feed_contains_global_session_scheduled_later_same_day() {
        let svc = SessionService::with_repos(
            Box::new(FakeSessionRepo {
                feed: vec![session_at("2099-05-17T20:00:00Z", "global")],
                mine: vec![],
                for_group: vec![],
            }),
            Box::new(FakeGroupRepo),
        );
        let result = svc.get_feed(&alice()).await.unwrap();
        assert_eq!(
            result.len(),
            1,
            "global session scheduled in the future must appear in feed"
        );
        assert_eq!(result[0].scheduled_at, "2099-05-17T20:00:00Z");
    }

    #[tokio::test]
    async fn mine_contains_session_scheduled_later_same_day() {
        let svc = SessionService::with_repos(
            Box::new(FakeSessionRepo {
                feed: vec![],
                mine: vec![session_at("2099-05-17T20:00:00Z", "global")],
                for_group: vec![],
            }),
            Box::new(FakeGroupRepo),
        );
        let result = svc.get_mine(&alice()).await.unwrap();
        assert_eq!(
            result.len(),
            1,
            "own session scheduled in the future must appear in my sessions"
        );
        assert_eq!(result[0].scheduled_at, "2099-05-17T20:00:00Z");
    }

    #[tokio::test]
    async fn group_feed_contains_session_scheduled_later_same_day() {
        let svc = SessionService::with_repos(
            Box::new(FakeSessionRepo {
                feed: vec![],
                mine: vec![],
                for_group: vec![session_at("2099-05-17T20:00:00Z", "groups")],
            }),
            Box::new(FakeGroupRepo),
        );
        let result = svc.get_for_group("g1", "u1").await.unwrap();
        assert_eq!(
            result.len(),
            1,
            "group session scheduled in the future must appear in group feed"
        );
        assert_eq!(result[0].scheduled_at, "2099-05-17T20:00:00Z");
    }

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
            notes: None,
        }
    }

    // ── validate_create_request ────────────────────────────────────────────────

    #[test]
    fn empty_game_name_is_rejected() {
        assert!(
            validate_create_request(&req("", "2099-12-31T18:00:00Z", "global", vec![])).is_err()
        );
    }

    #[test]
    fn whitespace_only_game_name_is_rejected() {
        assert!(
            validate_create_request(&req("   ", "2099-12-31T18:00:00Z", "global", vec![])).is_err()
        );
    }

    #[test]
    fn empty_datetime_is_rejected() {
        assert!(validate_create_request(&req("CS2", "", "global", vec![])).is_err());
    }

    #[test]
    fn invalid_scope_is_rejected() {
        assert!(
            validate_create_request(&req("CS2", "2099-12-31T18:00:00Z", "public", vec![])).is_err()
        );
    }

    #[test]
    fn groups_scope_without_group_ids_is_rejected() {
        assert!(
            validate_create_request(&req("CS2", "2099-12-31T18:00:00Z", "groups", vec![])).is_err()
        );
    }

    #[test]
    fn global_scope_is_valid() {
        assert!(
            validate_create_request(&req("CS2", "2099-12-31T18:00:00Z", "global", vec![])).is_ok()
        );
    }

    #[test]
    fn groups_scope_with_group_id_is_valid() {
        assert!(
            validate_create_request(&req(
                "CS2",
                "2099-12-31T18:00:00Z",
                "groups",
                vec!["group1"]
            ))
            .is_ok()
        );
    }

    // ── set_rsvp ──────────────────────────────────────────────────────────────

    struct RsvpFakeSessionRepo {
        result: Option<Session>,
    }

    #[async_trait]
    impl SessionRepo for RsvpFakeSessionRepo {
        async fn create(
            &self,
            _: String,
            _: String,
            _: String,
            _: String,
            _: String,
            _: Vec<String>,
            _: Option<String>,
        ) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_feed(&self, _: Vec<String>) -> Result<Vec<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_mine(&self, _: String) -> Result<Vec<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_for_group(&self, _: String) -> Result<Vec<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_by_id(&self, _: String) -> Result<Option<SessionDetail>, surrealdb::Error> {
            unimplemented!()
        }
        async fn join(&self, _: String, _: String) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn leave(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
        async fn set_rsvp(
            &self,
            _: String,
            _: String,
            _: String,
            _: String,
        ) -> Result<Option<Session>, surrealdb::Error> {
            Ok(self.result.clone())
        }
        async fn remove_rsvp(
            &self,
            _: String,
            _: String,
        ) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn delete(&self, _: String, _: String) -> Result<Vec<String>, surrealdb::Error> {
            unimplemented!()
        }
        async fn delete_as_admin(&self, _: String) -> Result<Vec<String>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_sessions_to_notify(
            &self,
        ) -> Result<Vec<crate::model::session::SessionStartReminder>, surrealdb::Error> {
            unimplemented!()
        }
        async fn mark_start_notified(&self, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
    }

    fn rsvp_svc(result: Option<Session>) -> SessionService {
        SessionService::with_repos(
            Box::new(RsvpFakeSessionRepo { result }),
            Box::new(FakeGroupRepo),
        )
    }

    fn bob() -> AuthUser {
        AuthUser {
            keycloak_id: "u2".into(),
            username: "Bob".into(),
            is_admin: false,
        }
    }

    fn session_with_participant(participant_id: &str) -> Session {
        Session {
            id: "s1".into(),
            user_id: "u1".into(),
            username: "Alice".into(),
            game: "CS2".into(),
            scheduled_at: "2099-05-17T20:00:00Z".into(),
            scope: "global".into(),
            group_ids: vec![],
            participants: vec![participant_id.into()],
            group_names: vec![],
            game_has_thumbnail: false,
            notes: None,
            rsvps: vec![],
        }
    }

    #[tokio::test]
    async fn rsvp_invalid_status_is_rejected() {
        let svc = rsvp_svc(None);
        let err = svc
            .set_rsvp("s1", &bob(), "pending".into())
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn rsvp_creator_gets_none() {
        // Repo returns None when WHERE user_id != $user_id is not satisfied
        let svc = rsvp_svc(None);
        let result = svc
            .set_rsvp("s1", &alice(), "accepted".into())
            .await
            .unwrap();
        assert!(
            result.is_none(),
            "creator must not be able to RSVP their own session"
        );
    }

    #[tokio::test]
    async fn rsvp_accepted_returns_session_response() {
        let svc = rsvp_svc(Some(session_with_participant("u2")));
        let result = svc.set_rsvp("s1", &bob(), "accepted".into()).await.unwrap();
        assert!(result.is_some());
        let resp = result.unwrap();
        // participant_count = 1 (creator) + participants.len()
        assert_eq!(resp.participant_count, 2);
        assert!(
            resp.is_participant,
            "bob must be flagged as participant after accepting"
        );
    }

    #[tokio::test]
    async fn rsvp_maybe_does_not_add_to_participants() {
        // Repo returns session without bob in participants (he only RSVPd maybe)
        let svc = rsvp_svc(Some(session_at("2099-05-17T20:00:00Z", "global")));
        let result = svc.set_rsvp("s1", &bob(), "maybe".into()).await.unwrap();
        assert!(result.is_some());
        let resp = result.unwrap();
        assert!(
            !resp.is_participant,
            "maybe RSVP must not mark user as participant"
        );
        assert_eq!(resp.participant_count, 1); // creator only
    }

    #[tokio::test]
    async fn rsvp_declined_does_not_add_to_participants() {
        let svc = rsvp_svc(Some(session_at("2099-05-17T20:00:00Z", "global")));
        let result = svc.set_rsvp("s1", &bob(), "declined".into()).await.unwrap();
        assert!(result.is_some());
        let resp = result.unwrap();
        assert!(!resp.is_participant);
    }

    // ── delete admin bypass ───────────────────────────────────────────────────

    struct AdminDeleteRepo;
    struct OwnerDeleteRepo;

    #[async_trait]
    impl SessionRepo for AdminDeleteRepo {
        async fn delete_as_admin(&self, _: String) -> Result<Vec<String>, surrealdb::Error> {
            Ok(vec!["g1".into()])
        }
        async fn delete(&self, _: String, _: String) -> Result<Vec<String>, surrealdb::Error> {
            panic!("delete called instead of delete_as_admin")
        }
        async fn create(
            &self,
            _: String,
            _: String,
            _: String,
            _: String,
            _: String,
            _: Vec<String>,
            _: Option<String>,
        ) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_feed(&self, _: Vec<String>) -> Result<Vec<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_mine(&self, _: String) -> Result<Vec<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_for_group(&self, _: String) -> Result<Vec<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_by_id(&self, _: String) -> Result<Option<SessionDetail>, surrealdb::Error> {
            unimplemented!()
        }
        async fn join(&self, _: String, _: String) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn leave(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
        async fn set_rsvp(
            &self,
            _: String,
            _: String,
            _: String,
            _: String,
        ) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn remove_rsvp(
            &self,
            _: String,
            _: String,
        ) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_sessions_to_notify(
            &self,
        ) -> Result<Vec<crate::model::session::SessionStartReminder>, surrealdb::Error> {
            unimplemented!()
        }
        async fn mark_start_notified(&self, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl SessionRepo for OwnerDeleteRepo {
        async fn delete(&self, _: String, _: String) -> Result<Vec<String>, surrealdb::Error> {
            Ok(vec!["g1".into()])
        }
        async fn delete_as_admin(&self, _: String) -> Result<Vec<String>, surrealdb::Error> {
            panic!("delete_as_admin called instead of delete")
        }
        async fn create(
            &self,
            _: String,
            _: String,
            _: String,
            _: String,
            _: String,
            _: Vec<String>,
            _: Option<String>,
        ) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_feed(&self, _: Vec<String>) -> Result<Vec<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_mine(&self, _: String) -> Result<Vec<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_for_group(&self, _: String) -> Result<Vec<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_by_id(&self, _: String) -> Result<Option<SessionDetail>, surrealdb::Error> {
            unimplemented!()
        }
        async fn join(&self, _: String, _: String) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn leave(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
        async fn set_rsvp(
            &self,
            _: String,
            _: String,
            _: String,
            _: String,
        ) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn remove_rsvp(
            &self,
            _: String,
            _: String,
        ) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_sessions_to_notify(
            &self,
        ) -> Result<Vec<crate::model::session::SessionStartReminder>, surrealdb::Error> {
            unimplemented!()
        }
        async fn mark_start_notified(&self, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn delete_with_admin_flag_calls_delete_as_admin() {
        let svc = SessionService::with_repos(Box::new(AdminDeleteRepo), Box::new(FakeGroupRepo));
        let group_ids = svc.delete("s1", "some_other_user", true).await.unwrap();
        assert_eq!(group_ids, vec!["g1"]);
    }

    #[tokio::test]
    async fn delete_without_admin_flag_calls_owner_delete() {
        let svc = SessionService::with_repos(Box::new(OwnerDeleteRepo), Box::new(FakeGroupRepo));
        let group_ids = svc.delete("s1", "u1", false).await.unwrap();
        assert_eq!(group_ids, vec!["g1"]);
    }
}
