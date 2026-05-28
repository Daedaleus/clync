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
