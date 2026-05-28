use std::sync::Arc;

use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::model::session_invitation::SessionInvitationRecord;
use crate::model::user::UserSummary;
use crate::repository::{
    session::SessionRepository,
    session_invitation::SessionInvitationRepository,
    traits::{SessionInvitationRepo, SessionRepo, UserRepo},
    user::UserRepository,
};

pub struct SessionInvitationService {
    inv_repo: Box<dyn SessionInvitationRepo>,
    session_repo: Box<dyn SessionRepo>,
    user_repo: Box<dyn UserRepo>,
}

impl SessionInvitationService {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self {
            inv_repo: Box::new(SessionInvitationRepository::new(Arc::clone(&db))),
            session_repo: Box::new(SessionRepository::new(Arc::clone(&db))),
            user_repo: Box::new(UserRepository::new(db)),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_repos(
        inv_repo: Box<dyn crate::repository::traits::SessionInvitationRepo>,
        session_repo: Box<dyn crate::repository::traits::SessionRepo>,
        user_repo: Box<dyn crate::repository::traits::UserRepo>,
    ) -> Self {
        Self {
            inv_repo,
            session_repo,
            user_repo,
        }
    }

    /// Returns the session's game name — used by the controller to build push notification text.
    pub async fn game_name_for_notify(&self, session_id: &str) -> Result<String, AppError> {
        Ok(self
            .session_repo
            .find_by_id(session_id.to_owned())
            .await?
            .map(|s| s.game)
            .unwrap_or_default())
    }

    /// Invite a mutual friend to a session.
    /// Any current participant (creator or joined) may invite.
    pub async fn invite(
        &self,
        session_id: &str,
        inviter: &AuthUser,
        invitee_id: &str,
    ) -> Result<(), AppError> {
        let session = self
            .session_repo
            .find_by_id(session_id.to_owned())
            .await?
            .ok_or_else(|| AppError::Validation("error.session.not_found".into()))?;

        let is_creator = session.user_id == inviter.keycloak_id;
        let is_participant = session
            .participants
            .iter()
            .any(|p| p.keycloak_id == inviter.keycloak_id);

        if !is_creator && !is_participant {
            return Err(AppError::Validation(
                "error.session_invitation.only_participants_can_invite".into(),
            ));
        }

        let all_participant_ids: Vec<String> = std::iter::once(session.user_id.clone())
            .chain(session.participants.iter().map(|p| p.keycloak_id.clone()))
            .collect();

        if all_participant_ids.contains(&invitee_id.to_owned()) {
            return Err(AppError::Validation(
                "error.session_invitation.already_participant".into(),
            ));
        }

        let inviter_friends = self
            .user_repo
            .get_friend_ids(inviter.keycloak_id.clone())
            .await?;
        if !inviter_friends.contains(&invitee_id.to_owned()) {
            return Err(AppError::Validation(
                "error.session_invitation.not_mutual_friends".into(),
            ));
        }
        let mutual = self
            .user_repo
            .find_who_added(inviter.keycloak_id.clone(), vec![invitee_id.to_owned()])
            .await?;
        if mutual.is_empty() {
            return Err(AppError::Validation(
                "error.session_invitation.not_mutual_friends".into(),
            ));
        }

        if self
            .inv_repo
            .exists_for_session_and_invitee(session_id.to_owned(), invitee_id.to_owned())
            .await?
        {
            return Err(AppError::Validation("error.invitation.already_sent".into()));
        }

        self.inv_repo
            .create(
                session_id.to_owned(),
                session.game,
                session.scheduled_at,
                inviter.keycloak_id.clone(),
                inviter.username.clone(),
                invitee_id.to_owned(),
            )
            .await?;
        Ok(())
    }

    pub async fn get_pending(
        &self,
        user_id: &str,
    ) -> Result<Vec<SessionInvitationRecord>, AppError> {
        Ok(self
            .inv_repo
            .find_pending_for_user(user_id.to_owned())
            .await?)
    }

    pub async fn accept(&self, inv_id: &str, user: &AuthUser) -> Result<(), AppError> {
        let inv = self
            .inv_repo
            .find_by_id_for_invitee(inv_id.to_owned(), user.keycloak_id.clone())
            .await?
            .ok_or_else(|| AppError::Validation("error.invitation.not_found".into()))?;

        self.session_repo
            .set_rsvp(
                inv.session_id,
                user.keycloak_id.clone(),
                user.username.clone(),
                "accepted".to_owned(),
            )
            .await?
            .ok_or_else(|| AppError::Validation("error.session.no_longer_available".into()))?;

        self.inv_repo
            .delete(inv_id.to_owned(), user.keycloak_id.clone())
            .await?;
        Ok(())
    }

    pub async fn decline(&self, inv_id: &str, user_id: &str) -> Result<(), AppError> {
        self.inv_repo
            .delete(inv_id.to_owned(), user_id.to_owned())
            .await?;
        Ok(())
    }

    /// Returns mutual friends of the requester who are not yet in the session
    /// and have no pending invitation.
    pub async fn get_invitable(
        &self,
        session_id: &str,
        requester: &AuthUser,
    ) -> Result<Vec<UserSummary>, AppError> {
        let session = self
            .session_repo
            .find_by_id(session_id.to_owned())
            .await?
            .ok_or_else(|| AppError::Validation("error.session.not_found".into()))?;

        let is_creator = session.user_id == requester.keycloak_id;
        let is_participant = session
            .participants
            .iter()
            .any(|p| p.keycloak_id == requester.keycloak_id);

        if !is_creator && !is_participant {
            return Err(AppError::Validation(
                "error.session_invitation.only_participants_see_list".into(),
            ));
        }

        let all_participant_ids: Vec<String> = std::iter::once(session.user_id)
            .chain(session.participants.into_iter().map(|p| p.keycloak_id))
            .collect();

        let friend_ids = self
            .user_repo
            .get_friend_ids(requester.keycloak_id.clone())
            .await?;
        if friend_ids.is_empty() {
            return Ok(vec![]);
        }

        let non_participants: Vec<String> = friend_ids
            .into_iter()
            .filter(|id| !all_participant_ids.contains(id))
            .collect();
        if non_participants.is_empty() {
            return Ok(vec![]);
        }

        let mutual = self
            .user_repo
            .find_who_added(requester.keycloak_id.clone(), non_participants)
            .await?;
        if mutual.is_empty() {
            return Ok(vec![]);
        }

        let mut result = Vec::new();
        for user in mutual {
            let already_invited = self
                .inv_repo
                .exists_for_session_and_invitee(session_id.to_owned(), user.keycloak_id.clone())
                .await?;
            if !already_invited {
                result.push(user);
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::{
        middleware::auth::AuthUser,
        model::{
            session::{ParticipantRecord, Session, SessionDetail, SessionStartReminder},
            session_invitation::SessionInvitationRecord,
            user::{User, UserFullProfile, UserSummary, Visibility},
        },
        repository::traits::{SessionInvitationRepo, SessionRepo, UserRepo},
    };

    // ── Fakes ─────────────────────────────────────────────────────────────────

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

    fn session_detail(creator_id: &str, participant_ids: &[&str]) -> SessionDetail {
        SessionDetail {
            id: "s1".into(),
            user_id: creator_id.into(),
            username: "Alice".into(),
            game: "CS2".into(),
            scheduled_at: "2099-01-01T18:00:00Z".into(),
            scope: "global".into(),
            group_ids: vec![],
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
        detail: Option<SessionDetail>,
    }

    #[async_trait]
    impl SessionRepo for FakeSessionRepo {
        async fn find_by_id(&self, _: String) -> Result<Option<SessionDetail>, surrealdb::Error> {
            Ok(self.detail.clone())
        }
        async fn set_rsvp(
            &self,
            _: String,
            _: String,
            _: String,
            _: String,
        ) -> Result<Option<Session>, surrealdb::Error> {
            Ok(None)
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
        ) -> Result<Vec<SessionStartReminder>, surrealdb::Error> {
            unimplemented!()
        }
        async fn mark_start_notified(&self, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
    }

    struct FakeUserRepo {
        friend_ids: Vec<String>,
        who_added: Vec<UserSummary>,
    }

    #[async_trait]
    impl UserRepo for FakeUserRepo {
        async fn get_friend_ids(&self, _: String) -> Result<Vec<String>, surrealdb::Error> {
            Ok(self.friend_ids.clone())
        }
        async fn find_who_added(
            &self,
            _: String,
            _: Vec<String>,
        ) -> Result<Vec<UserSummary>, surrealdb::Error> {
            Ok(self.who_added.clone())
        }
        async fn upsert(&self, _: User) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
        async fn search(&self, _: String, _: String) -> Result<Vec<UserSummary>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_by_ids(&self, _: Vec<String>) -> Result<Vec<UserSummary>, surrealdb::Error> {
            unimplemented!()
        }
        async fn add_game(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
        async fn remove_game(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
        async fn add_friend(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
        async fn remove_friend(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
        async fn get_full_profile(
            &self,
            _: String,
        ) -> Result<Option<UserFullProfile>, surrealdb::Error> {
            unimplemented!()
        }
        async fn update_social_profile(
            &self,
            _: String,
            _: Option<String>,
            _: Visibility,
            _: Option<String>,
            _: Visibility,
        ) -> Result<(), surrealdb::Error> {
            unimplemented!()
        }
    }

    struct FakeSessionInvitationRepo {
        exists: bool,
    }

    #[async_trait]
    impl SessionInvitationRepo for FakeSessionInvitationRepo {
        async fn exists_for_session_and_invitee(
            &self,
            _: String,
            _: String,
        ) -> Result<bool, surrealdb::Error> {
            Ok(self.exists)
        }
        async fn create(
            &self,
            _: String,
            _: String,
            _: String,
            _: String,
            _: String,
            _: String,
        ) -> Result<(), surrealdb::Error> {
            Ok(())
        }
        async fn find_pending_for_user(
            &self,
            _: String,
        ) -> Result<Vec<SessionInvitationRecord>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_by_id_for_invitee(
            &self,
            _: String,
            _: String,
        ) -> Result<Option<SessionInvitationRecord>, surrealdb::Error> {
            unimplemented!()
        }
        async fn delete(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            Ok(())
        }
    }

    fn svc(
        detail: Option<SessionDetail>,
        friend_ids: Vec<&str>,
        who_added: Vec<UserSummary>,
        inv_exists: bool,
    ) -> SessionInvitationService {
        SessionInvitationService::with_repos(
            Box::new(FakeSessionInvitationRepo { exists: inv_exists }),
            Box::new(FakeSessionRepo { detail }),
            Box::new(FakeUserRepo {
                friend_ids: friend_ids.into_iter().map(String::from).collect(),
                who_added,
            }),
        )
    }

    fn summary(id: &str) -> UserSummary {
        UserSummary {
            keycloak_id: id.into(),
            username: id.into(),
        }
    }

    // ── SessionInvitationService::invite ──────────────────────────────────────

    #[tokio::test]
    async fn invite_session_not_found_returns_error() {
        let err = svc(None, vec![], vec![], false)
            .invite("s1", &alice(), "u3")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn invite_non_participant_is_rejected() {
        // Bob (u2) is neither creator (u1) nor a participant
        let err = svc(Some(session_detail("u1", &[])), vec![], vec![], false)
            .invite("s1", &bob(), "u3")
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(ref msg) if msg.contains("only_participants_can_invite"))
        );
    }

    #[tokio::test]
    async fn invite_invitee_already_participant_is_rejected() {
        // u2 is a participant; trying to invite u2 again
        let err = svc(
            Some(session_detail("u1", &["u2"])),
            vec!["u2"],
            vec![summary("u2")],
            false,
        )
        .invite("s1", &alice(), "u2")
        .await
        .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(ref msg) if msg.contains("already_participant"))
        );
    }

    #[tokio::test]
    async fn invite_invitee_is_creator_is_rejected() {
        // Trying to invite the session creator (u1) to their own session
        let err = svc(
            Some(session_detail("u1", &[])),
            vec!["u1"],
            vec![summary("u1")],
            false,
        )
        .invite("s1", &alice(), "u1")
        .await
        .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(ref msg) if msg.contains("already_participant"))
        );
    }

    #[tokio::test]
    async fn invite_not_mutual_friends_is_rejected() {
        // Alice (creator) has added u3 but u3 hasn't added Alice back
        let err = svc(Some(session_detail("u1", &[])), vec!["u3"], vec![], false)
            .invite("s1", &alice(), "u3")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(ref msg) if msg.contains("not_mutual_friends")));
    }

    #[tokio::test]
    async fn invite_already_sent_is_rejected() {
        let err = svc(
            Some(session_detail("u1", &[])),
            vec!["u3"],
            vec![summary("u3")],
            true,
        )
        .invite("s1", &alice(), "u3")
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(ref msg) if msg.contains("already_sent")));
    }

    #[tokio::test]
    async fn invite_by_creator_to_mutual_friend_succeeds() {
        let result = svc(
            Some(session_detail("u1", &[])),
            vec!["u3"],
            vec![summary("u3")],
            false,
        )
        .invite("s1", &alice(), "u3")
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn invite_by_participant_to_mutual_friend_succeeds() {
        // Bob (u2) is a participant and invites u3
        let result = svc(
            Some(session_detail("u1", &["u2"])),
            vec!["u3"],
            vec![summary("u3")],
            false,
        )
        .invite("s1", &bob(), "u3")
        .await;
        assert!(result.is_ok());
    }

    // ── SessionInvitationService::get_invitable ───────────────────────────────

    #[tokio::test]
    async fn get_invitable_non_participant_is_rejected() {
        // Bob is not a participant
        let err = svc(Some(session_detail("u1", &[])), vec![], vec![], false)
            .get_invitable("s1", &bob())
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(ref msg) if msg.contains("only_participants_see_list"))
        );
    }

    #[tokio::test]
    async fn get_invitable_no_friends_returns_empty() {
        let result = svc(Some(session_detail("u1", &[])), vec![], vec![], false)
            .get_invitable("s1", &alice())
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn get_invitable_filters_existing_participants() {
        // u2 is a friend of Alice but already a participant
        let result = svc(
            Some(session_detail("u1", &["u2"])),
            vec!["u2"],
            vec![summary("u2")],
            false,
        )
        .get_invitable("s1", &alice())
        .await
        .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn get_invitable_returns_mutual_non_participants() {
        let result = svc(
            Some(session_detail("u1", &[])),
            vec!["u3"],
            vec![summary("u3")],
            false,
        )
        .get_invitable("s1", &alice())
        .await
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].keycloak_id, "u3");
    }
}
