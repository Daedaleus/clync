use std::sync::Arc;

use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::dto::session::{CreateSessionRequest, SessionDetailResponse, SessionResponse};
use crate::error::{AppError, codes};
use crate::middleware::auth::AuthUser;
use crate::model::session::{RsvpStatus, SessionScope};
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
                req.scope.to_string(),
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

    /// Returns None for both "not found" and "group-scoped with no access" (security: don't reveal existence).
    pub async fn get_detail(
        &self,
        session_id: &str,
        requester_id: &str,
    ) -> Result<Option<SessionDetailResponse>, AppError> {
        let Some(detail) = self.repo.find_by_id(session_id.to_owned()).await? else {
            return Ok(None);
        };

        if detail.scope == "groups" {
            let is_creator = detail.user_id == requester_id;
            let is_participant = detail
                .participants
                .iter()
                .any(|p| p.keycloak_id == requester_id);
            if !is_creator && !is_participant {
                let user_groups = self
                    .group_repo
                    .find_by_member(requester_id.to_owned())
                    .await?;
                let is_member = user_groups.iter().any(|g| detail.group_ids.contains(&g.id));
                if !is_member {
                    return Ok(None);
                }
            }
        }

        Ok(Some(detail.into_detail_response(requester_id)))
    }

    /// Join a session directly. For group-scoped sessions the user must be a group member.
    pub async fn join(
        &self,
        session_id: &str,
        user: &AuthUser,
    ) -> Result<Option<SessionResponse>, AppError> {
        let detail = self
            .repo
            .find_by_id(session_id.to_owned())
            .await?
            .ok_or_else(|| AppError::Validation(codes::session::NOT_FOUND.into()))?;

        if detail.scope == "groups" {
            self.require_group_access(&user.keycloak_id, &detail)
                .await?;
        }

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

    /// Set RSVP status. For group-scoped sessions the user must be a group member or
    /// already a participant (e.g. accepted via session invitation).
    /// Returns updated session (for SSE broadcast), None if caller is the creator.
    pub async fn set_rsvp(
        &self,
        session_id: &str,
        user: &AuthUser,
        status: RsvpStatus,
    ) -> Result<Option<SessionResponse>, AppError> {
        let detail = self
            .repo
            .find_by_id(session_id.to_owned())
            .await?
            .ok_or_else(|| AppError::Validation(codes::session::NOT_FOUND.into()))?;

        if detail.scope == "groups" {
            let already_participant = detail.user_id == user.keycloak_id
                || detail
                    .participants
                    .iter()
                    .any(|p| p.keycloak_id == user.keycloak_id);
            if !already_participant {
                self.require_group_access(&user.keycloak_id, &detail)
                    .await?;
            }
        }

        let session = self
            .repo
            .set_rsvp(
                session_id.to_owned(),
                user.keycloak_id.clone(),
                user.username.clone(),
                status.to_string(),
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

    /// Returns Validation(NOT_FOUND) for group sessions where the user is not a member.
    /// Uses NOT_FOUND instead of Unauthorized to avoid leaking session existence.
    async fn require_group_access(
        &self,
        user_id: &str,
        detail: &crate::model::session::SessionDetail,
    ) -> Result<(), AppError> {
        let user_groups = self.group_repo.find_by_member(user_id.to_owned()).await?;
        let is_member = user_groups.iter().any(|g| detail.group_ids.contains(&g.id));
        if !is_member {
            return Err(AppError::Validation(codes::session::NOT_FOUND.into()));
        }
        Ok(())
    }
}

pub(crate) fn validate_create_request(req: &CreateSessionRequest) -> Result<(), AppError> {
    if req.game.trim().is_empty() {
        return Err(AppError::Validation(codes::game::NAME_REQUIRED.into()));
    }
    if req.scheduled_at.is_empty() {
        return Err(AppError::Validation(codes::session::DATE_REQUIRED.into()));
    }
    let scheduled = chrono::DateTime::parse_from_rfc3339(&req.scheduled_at)
        .map_err(|_| AppError::Validation(codes::session::INVALID_DATE.into()))?;
    if scheduled <= chrono::Utc::now() {
        return Err(AppError::Validation(codes::session::DATE_IN_PAST.into()));
    }
    if req.scope == SessionScope::Groups && req.group_ids.is_empty() {
        return Err(AppError::Validation(codes::session::GROUPS_REQUIRED.into()));
    }
    if let Some(notes) = &req.notes
        && notes.trim().len() > 500
    {
        return Err(AppError::Validation(codes::session::NOTES_TOO_LONG.into()));
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
            session::{ParticipantRecord, Session, SessionDetail},
        },
        repository::traits::{GroupRepo, SessionRepo},
    };

    // ── Shared fakes ──────────────────────────────────────────────────────────

    fn global_detail() -> SessionDetail {
        SessionDetail {
            id: "s1".into(),
            user_id: "u1".into(),
            username: "Alice".into(),
            game: "CS2".into(),
            scheduled_at: "2099-01-01T18:00:00Z".into(),
            scope: "global".into(),
            group_ids: vec![],
            group_names: vec![],
            game_has_thumbnail: false,
            participants: vec![],
            notes: None,
            rsvps: vec![],
        }
    }

    fn group_detail(participant_ids: &[&str]) -> SessionDetail {
        SessionDetail {
            id: "s1".into(),
            user_id: "u1".into(),
            username: "Alice".into(),
            game: "CS2".into(),
            scheduled_at: "2099-01-01T18:00:00Z".into(),
            scope: "groups".into(),
            group_ids: vec!["g1".into()],
            group_names: vec![],
            game_has_thumbnail: false,
            participants: participant_ids
                .iter()
                .map(|id| ParticipantRecord {
                    keycloak_id: id.to_string(),
                    username: id.to_string(),
                })
                .collect(),
            notes: None,
            rsvps: vec![],
        }
    }

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

    struct FakeGroupRepo {
        member_groups: Vec<Group>,
    }

    impl FakeGroupRepo {
        fn no_groups() -> Self {
            Self {
                member_groups: vec![],
            }
        }
        fn with_group(group_id: &str) -> Self {
            Self {
                member_groups: vec![Group {
                    id: group_id.into(),
                    name: "Test".into(),
                    is_public: false,
                    members: vec![],
                    creator_id: None,
                    discord_invite: None,
                }],
            }
        }
    }

    #[async_trait]
    impl GroupRepo for FakeGroupRepo {
        async fn find_by_member(&self, _: String) -> Result<Vec<Group>, surrealdb::Error> {
            Ok(self.member_groups.clone())
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

    fn bob() -> AuthUser {
        AuthUser {
            keycloak_id: "u2".into(),
            username: "Bob".into(),
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
            Box::new(FakeGroupRepo::no_groups()),
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
            Box::new(FakeGroupRepo::no_groups()),
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
            Box::new(FakeGroupRepo::no_groups()),
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
        scope: SessionScope,
        group_ids: Vec<&str>,
    ) -> CreateSessionRequest {
        CreateSessionRequest {
            game: game.into(),
            scheduled_at: scheduled_at.into(),
            scope,
            group_ids: group_ids.into_iter().map(String::from).collect(),
            notes: None,
        }
    }

    // ── validate_create_request ────────────────────────────────────────────────

    #[test]
    fn empty_game_name_is_rejected() {
        assert!(
            validate_create_request(&req(
                "",
                "2099-12-31T18:00:00Z",
                SessionScope::Global,
                vec![]
            ))
            .is_err()
        );
    }

    #[test]
    fn whitespace_only_game_name_is_rejected() {
        assert!(
            validate_create_request(&req(
                "   ",
                "2099-12-31T18:00:00Z",
                SessionScope::Global,
                vec![]
            ))
            .is_err()
        );
    }

    #[test]
    fn empty_datetime_is_rejected() {
        assert!(validate_create_request(&req("CS2", "", SessionScope::Global, vec![])).is_err());
    }

    #[test]
    fn groups_scope_without_group_ids_is_rejected() {
        assert!(
            validate_create_request(&req(
                "CS2",
                "2099-12-31T18:00:00Z",
                SessionScope::Groups,
                vec![]
            ))
            .is_err()
        );
    }

    #[test]
    fn global_scope_is_valid() {
        assert!(
            validate_create_request(&req(
                "CS2",
                "2099-12-31T18:00:00Z",
                SessionScope::Global,
                vec![]
            ))
            .is_ok()
        );
    }

    #[test]
    fn groups_scope_with_group_id_is_valid() {
        assert!(
            validate_create_request(&req(
                "CS2",
                "2099-12-31T18:00:00Z",
                SessionScope::Groups,
                vec!["group1"]
            ))
            .is_ok()
        );
    }

    // ── set_rsvp ──────────────────────────────────────────────────────────────

    struct RsvpFakeSessionRepo {
        rsvp_result: Option<Session>,
    }

    #[async_trait]
    impl SessionRepo for RsvpFakeSessionRepo {
        async fn find_by_id(&self, _: String) -> Result<Option<SessionDetail>, surrealdb::Error> {
            Ok(Some(global_detail()))
        }
        async fn set_rsvp(
            &self,
            _: String,
            _: String,
            _: String,
            _: String,
        ) -> Result<Option<Session>, surrealdb::Error> {
            Ok(self.rsvp_result.clone())
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
        async fn join(&self, _: String, _: String) -> Result<Option<Session>, surrealdb::Error> {
            unimplemented!()
        }
        async fn leave(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
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

    fn rsvp_svc(result: Option<Session>) -> SessionService {
        SessionService::with_repos(
            Box::new(RsvpFakeSessionRepo {
                rsvp_result: result,
            }),
            Box::new(FakeGroupRepo::no_groups()),
        )
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
    async fn rsvp_creator_gets_none() {
        let svc = rsvp_svc(None);
        let result = svc
            .set_rsvp("s1", &alice(), RsvpStatus::Accepted)
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
        let result = svc
            .set_rsvp("s1", &bob(), RsvpStatus::Accepted)
            .await
            .unwrap();
        assert!(result.is_some());
        let resp = result.unwrap();
        assert_eq!(resp.participant_count, 2);
        assert!(
            resp.is_participant,
            "bob must be flagged as participant after accepting"
        );
    }

    #[tokio::test]
    async fn rsvp_maybe_does_not_add_to_participants() {
        let svc = rsvp_svc(Some(session_at("2099-05-17T20:00:00Z", "global")));
        let result = svc.set_rsvp("s1", &bob(), RsvpStatus::Maybe).await.unwrap();
        assert!(result.is_some());
        let resp = result.unwrap();
        assert!(
            !resp.is_participant,
            "maybe RSVP must not mark user as participant"
        );
        assert_eq!(resp.participant_count, 1);
    }

    #[tokio::test]
    async fn rsvp_declined_does_not_add_to_participants() {
        let svc = rsvp_svc(Some(session_at("2099-05-17T20:00:00Z", "global")));
        let result = svc
            .set_rsvp("s1", &bob(), RsvpStatus::Declined)
            .await
            .unwrap();
        assert!(result.is_some());
        assert!(!result.unwrap().is_participant);
    }

    // ── Security: group-scoped session access control ─────────────────────────

    struct SecurityRepo {
        detail: SessionDetail,
        join_result: Option<Session>,
        rsvp_result: Option<Session>,
    }

    #[async_trait]
    impl SessionRepo for SecurityRepo {
        async fn find_by_id(&self, _: String) -> Result<Option<SessionDetail>, surrealdb::Error> {
            Ok(Some(self.detail.clone()))
        }
        async fn join(&self, _: String, _: String) -> Result<Option<Session>, surrealdb::Error> {
            Ok(self.join_result.clone())
        }
        async fn set_rsvp(
            &self,
            _: String,
            _: String,
            _: String,
            _: String,
        ) -> Result<Option<Session>, surrealdb::Error> {
            Ok(self.rsvp_result.clone())
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
        async fn leave(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
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

    fn security_svc(detail: SessionDetail, is_member: bool) -> SessionService {
        let group_repo: Box<dyn GroupRepo> = if is_member {
            Box::new(FakeGroupRepo::with_group("g1"))
        } else {
            Box::new(FakeGroupRepo::no_groups())
        };
        SessionService::with_repos(
            Box::new(SecurityRepo {
                detail,
                join_result: Some(session_at("2099-01-01T18:00:00Z", "groups")),
                rsvp_result: Some(session_at("2099-01-01T18:00:00Z", "groups")),
            }),
            group_repo,
        )
    }

    #[tokio::test]
    async fn join_group_session_without_membership_is_rejected() {
        let err = security_svc(group_detail(&[]), false)
            .join("s1", &bob())
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn join_group_session_as_member_succeeds() {
        let result = security_svc(group_detail(&[]), true)
            .join("s1", &bob())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn rsvp_group_session_without_membership_is_rejected() {
        let err = security_svc(group_detail(&[]), false)
            .set_rsvp("s1", &bob(), RsvpStatus::Accepted)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn rsvp_group_session_as_existing_participant_succeeds() {
        // Bob was invited via session invitation and is already a participant
        let result = security_svc(group_detail(&["u2"]), false)
            .set_rsvp("s1", &bob(), RsvpStatus::Maybe)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_detail_group_session_without_access_returns_none() {
        let result = security_svc(group_detail(&[]), false)
            .get_detail("s1", "u2")
            .await
            .unwrap();
        assert!(
            result.is_none(),
            "non-member non-participant must not see group session"
        );
    }

    #[tokio::test]
    async fn get_detail_group_session_as_member_returns_detail() {
        let result = security_svc(group_detail(&[]), true)
            .get_detail("s1", "u2")
            .await
            .unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn get_detail_group_session_as_participant_returns_detail() {
        // Bob (u2) is a participant (invited) but not a group member
        let result = security_svc(group_detail(&["u2"]), false)
            .get_detail("s1", "u2")
            .await
            .unwrap();
        assert!(
            result.is_some(),
            "invited participant must be able to see the session"
        );
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
        let svc = SessionService::with_repos(
            Box::new(AdminDeleteRepo),
            Box::new(FakeGroupRepo::no_groups()),
        );
        let group_ids = svc.delete("s1", "some_other_user", true).await.unwrap();
        assert_eq!(group_ids, vec!["g1"]);
    }

    #[tokio::test]
    async fn delete_without_admin_flag_calls_owner_delete() {
        let svc = SessionService::with_repos(
            Box::new(OwnerDeleteRepo),
            Box::new(FakeGroupRepo::no_groups()),
        );
        let group_ids = svc.delete("s1", "u1", false).await.unwrap();
        assert_eq!(group_ids, vec!["g1"]);
    }
}
