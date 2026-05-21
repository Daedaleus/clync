use std::sync::Arc;

use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::model::invitation::InvitationRecord;
use crate::model::user::UserSummary;
use crate::repository::{
    group::GroupRepository,
    invitation::InvitationRepository,
    traits::{GroupRepo, InvitationRepo, UserRepo},
    user::UserRepository,
};

pub struct InvitationService {
    inv_repo: Box<dyn InvitationRepo>,
    user_repo: Box<dyn UserRepo>,
    group_repo: Box<dyn GroupRepo>,
}

impl InvitationService {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self {
            inv_repo: Box::new(InvitationRepository::new(Arc::clone(&db))),
            user_repo: Box::new(UserRepository::new(Arc::clone(&db))),
            group_repo: Box::new(GroupRepository::new(db)),
        }
    }

    /// Invite a mutual friend to a group.
    /// Validates: inviter is member, invitee is mutual friend, invitee not already member, no duplicate.
    pub async fn invite(
        &self,
        group_id: &str,
        inviter: &AuthUser,
        invitee_id: &str,
    ) -> Result<(), AppError> {
        let group = self
            .group_repo
            .find_by_id(group_id.to_owned())
            .await?
            .ok_or_else(|| AppError::Validation("Gruppe nicht gefunden".into()))?;

        if !group.members.contains(&inviter.keycloak_id) {
            return Err(AppError::Validation(
                "Nur Gruppenmitglieder können einladen".into(),
            ));
        }
        if group.members.contains(&invitee_id.to_owned()) {
            return Err(AppError::Validation("Benutzer ist bereits Mitglied".into()));
        }

        // Mutual friendship check: inviter has added invitee AND invitee has added inviter
        let inviter_friends = self
            .user_repo
            .get_friend_ids(inviter.keycloak_id.clone())
            .await?;
        if !inviter_friends.contains(&invitee_id.to_owned()) {
            return Err(AppError::Validation(
                "Nur gegenseitige Freunde können eingeladen werden".into(),
            ));
        }
        let mutual = self
            .user_repo
            .find_who_added(inviter.keycloak_id.clone(), vec![invitee_id.to_owned()])
            .await?;
        if mutual.is_empty() {
            return Err(AppError::Validation(
                "Nur gegenseitige Freunde können eingeladen werden".into(),
            ));
        }

        if self
            .inv_repo
            .exists_for_group_and_invitee(group_id.to_owned(), invitee_id.to_owned())
            .await?
        {
            return Err(AppError::Validation("Einladung bereits versandt".into()));
        }

        self.inv_repo
            .create(
                group_id.to_owned(),
                group.name,
                inviter.keycloak_id.clone(),
                inviter.username.clone(),
                invitee_id.to_owned(),
            )
            .await?;
        Ok(())
    }

    pub async fn get_pending(&self, user_id: &str) -> Result<Vec<InvitationRecord>, AppError> {
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
            .ok_or_else(|| AppError::Validation("Einladung nicht gefunden".into()))?;

        self.group_repo
            .join(inv.group_id, user.keycloak_id.clone())
            .await?;
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

    /// Returns the group name — used by the controller to build push notification text.
    pub async fn group_name_for_notify(&self, group_id: &str) -> Result<String, AppError> {
        Ok(self
            .group_repo
            .find_by_id(group_id.to_owned())
            .await?
            .map(|g| g.name)
            .unwrap_or_default())
    }

    /// Returns mutual friends of the requester who are not yet members and have no pending invite.
    pub async fn get_invitable(
        &self,
        group_id: &str,
        requester: &AuthUser,
    ) -> Result<Vec<UserSummary>, AppError> {
        let group = self
            .group_repo
            .find_by_id(group_id.to_owned())
            .await?
            .ok_or_else(|| AppError::Validation("Gruppe nicht gefunden".into()))?;

        if !group.members.contains(&requester.keycloak_id) {
            return Err(AppError::Validation(
                "Nur Gruppenmitglieder sehen die Einladeliste".into(),
            ));
        }

        // Friends the requester has added
        let friend_ids = self
            .user_repo
            .get_friend_ids(requester.keycloak_id.clone())
            .await?;
        if friend_ids.is_empty() {
            return Ok(vec![]);
        }

        // Filter: not already a member
        let non_members: Vec<String> = friend_ids
            .into_iter()
            .filter(|id| !group.members.contains(id))
            .collect();
        if non_members.is_empty() {
            return Ok(vec![]);
        }

        // Keep only mutual friends (they have added the requester back)
        let mutual = self
            .user_repo
            .find_who_added(requester.keycloak_id.clone(), non_members)
            .await?;
        if mutual.is_empty() {
            return Ok(vec![]);
        }

        // Filter out those with a pending invitation already
        let mut result = Vec::new();
        for user in mutual {
            let already_invited = self
                .inv_repo
                .exists_for_group_and_invitee(group_id.to_owned(), user.keycloak_id.clone())
                .await?;
            if !already_invited {
                result.push(user);
            }
        }
        Ok(result)
    }
}
