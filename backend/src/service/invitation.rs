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

    #[cfg(test)]
    pub(crate) fn with_repos(
        inv_repo: Box<dyn crate::repository::traits::InvitationRepo>,
        user_repo: Box<dyn crate::repository::traits::UserRepo>,
        group_repo: Box<dyn crate::repository::traits::GroupRepo>,
    ) -> Self {
        Self {
            inv_repo,
            user_repo,
            group_repo,
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
            .ok_or_else(|| AppError::Validation("error.group.not_found".into()))?;

        if !group.members.contains(&inviter.keycloak_id) {
            return Err(AppError::Validation(
                "error.invitation.only_members_can_invite".into(),
            ));
        }
        if group.members.contains(&invitee_id.to_owned()) {
            return Err(AppError::Validation(
                "error.invitation.already_member".into(),
            ));
        }

        // Mutual friendship check: inviter has added invitee AND invitee has added inviter
        let inviter_friends = self
            .user_repo
            .get_friend_ids(inviter.keycloak_id.clone())
            .await?;
        if !inviter_friends.contains(&invitee_id.to_owned()) {
            return Err(AppError::Validation(
                "error.invitation.not_mutual_friends".into(),
            ));
        }
        let mutual = self
            .user_repo
            .find_who_added(inviter.keycloak_id.clone(), vec![invitee_id.to_owned()])
            .await?;
        if mutual.is_empty() {
            return Err(AppError::Validation(
                "error.invitation.not_mutual_friends".into(),
            ));
        }

        if self
            .inv_repo
            .exists_for_group_and_invitee(group_id.to_owned(), invitee_id.to_owned())
            .await?
        {
            return Err(AppError::Validation("error.invitation.already_sent".into()));
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
            .ok_or_else(|| AppError::Validation("error.invitation.not_found".into()))?;

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
            .ok_or_else(|| AppError::Validation("error.group.not_found".into()))?;

        if !group.members.contains(&requester.keycloak_id) {
            return Err(AppError::Validation(
                "error.invitation.only_members_see_list".into(),
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

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::{
        middleware::auth::AuthUser,
        model::{
            group::Group,
            invitation::InvitationRecord,
            user::{User, UserFullProfile, UserSummary, Visibility},
        },
        repository::traits::{GroupRepo, InvitationRepo, UserRepo},
    };

    // ── Fakes ─────────────────────────────────────────────────────────────────

    fn make_group(members: Vec<&str>) -> Group {
        Group {
            id: "g1".into(),
            name: "Test Group".into(),
            is_public: false,
            members: members.into_iter().map(String::from).collect(),
            creator_id: Some("u1".into()),
            discord_invite: None,
        }
    }

    fn alice() -> AuthUser {
        AuthUser {
            keycloak_id: "u1".into(),
            username: "Alice".into(),
            is_admin: false,
        }
    }

    struct FakeGroupRepo {
        group: Option<Group>,
    }

    #[async_trait]
    impl GroupRepo for FakeGroupRepo {
        async fn find_by_id(&self, _: String) -> Result<Option<Group>, surrealdb::Error> {
            Ok(self.group.clone())
        }
        async fn create(
            &self,
            _: String,
            _: bool,
            _: String,
        ) -> Result<Option<Group>, surrealdb::Error> {
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
        async fn find_by_member(&self, _: String) -> Result<Vec<Group>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_public_by_member(&self, _: String) -> Result<Vec<Group>, surrealdb::Error> {
            unimplemented!()
        }
        async fn join(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            Ok(())
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

    struct FakeInvitationRepo {
        exists: bool,
    }

    #[async_trait]
    impl InvitationRepo for FakeInvitationRepo {
        async fn exists_for_group_and_invitee(
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
        ) -> Result<(), surrealdb::Error> {
            Ok(())
        }
        async fn find_pending_for_user(
            &self,
            _: String,
        ) -> Result<Vec<InvitationRecord>, surrealdb::Error> {
            unimplemented!()
        }
        async fn find_by_id_for_invitee(
            &self,
            _: String,
            _: String,
        ) -> Result<Option<InvitationRecord>, surrealdb::Error> {
            unimplemented!()
        }
        async fn delete(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            Ok(())
        }
    }

    fn svc(
        group: Option<Group>,
        friend_ids: Vec<&str>,
        who_added: Vec<UserSummary>,
        inv_exists: bool,
    ) -> InvitationService {
        InvitationService::with_repos(
            Box::new(FakeInvitationRepo { exists: inv_exists }),
            Box::new(FakeUserRepo {
                friend_ids: friend_ids.into_iter().map(String::from).collect(),
                who_added,
            }),
            Box::new(FakeGroupRepo { group }),
        )
    }

    fn summary(id: &str) -> UserSummary {
        UserSummary {
            keycloak_id: id.into(),
            username: id.into(),
        }
    }

    // ── InvitationService::invite ─────────────────────────────────────────────

    #[tokio::test]
    async fn invite_group_not_found_returns_error() {
        let err = svc(None, vec![], vec![], false)
            .invite("g1", &alice(), "u2")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn invite_non_member_inviter_is_rejected() {
        // Alice (u1) is NOT in the group
        let err = svc(Some(make_group(vec!["u2"])), vec![], vec![], false)
            .invite("g1", &alice(), "u2")
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(ref msg) if msg.contains("only_members_can_invite"))
        );
    }

    #[tokio::test]
    async fn invite_invitee_already_member_is_rejected() {
        // Both u1 and u2 are members; can't invite u2 again
        let err = svc(
            Some(make_group(vec!["u1", "u2"])),
            vec!["u2"],
            vec![summary("u2")],
            false,
        )
        .invite("g1", &alice(), "u2")
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(ref msg) if msg.contains("already_member")));
    }

    #[tokio::test]
    async fn invite_inviter_has_not_added_invitee_is_rejected() {
        // Alice is member but has NOT added u2 as friend
        let err = svc(Some(make_group(vec!["u1"])), vec![], vec![], false)
            .invite("g1", &alice(), "u2")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(ref msg) if msg.contains("not_mutual_friends")));
    }

    #[tokio::test]
    async fn invite_not_mutual_friendship_is_rejected() {
        // Alice added u2 but u2 has NOT added Alice back (find_who_added returns empty)
        let err = svc(Some(make_group(vec!["u1"])), vec!["u2"], vec![], false)
            .invite("g1", &alice(), "u2")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(ref msg) if msg.contains("not_mutual_friends")));
    }

    #[tokio::test]
    async fn invite_already_sent_is_rejected() {
        let err = svc(
            Some(make_group(vec!["u1"])),
            vec!["u2"],
            vec![summary("u2")],
            true,
        )
        .invite("g1", &alice(), "u2")
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(ref msg) if msg.contains("already_sent")));
    }

    #[tokio::test]
    async fn invite_valid_succeeds() {
        let result = svc(
            Some(make_group(vec!["u1"])),
            vec!["u2"],
            vec![summary("u2")],
            false,
        )
        .invite("g1", &alice(), "u2")
        .await;
        assert!(result.is_ok());
    }

    // ── InvitationService::get_invitable ──────────────────────────────────────

    #[tokio::test]
    async fn get_invitable_non_member_is_rejected() {
        // Alice (u1) is NOT in the group
        let err = svc(Some(make_group(vec!["u2"])), vec![], vec![], false)
            .get_invitable("g1", &alice())
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(ref msg) if msg.contains("only_members_see_list"))
        );
    }

    #[tokio::test]
    async fn get_invitable_no_friends_returns_empty() {
        let result = svc(Some(make_group(vec!["u1"])), vec![], vec![], false)
            .get_invitable("g1", &alice())
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn get_invitable_all_friends_already_members_returns_empty() {
        // u2 is a friend but already a member
        let result = svc(
            Some(make_group(vec!["u1", "u2"])),
            vec!["u2"],
            vec![summary("u2")],
            false,
        )
        .get_invitable("g1", &alice())
        .await
        .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn get_invitable_already_invited_are_excluded() {
        // u2 is a mutual friend, not yet a member, but already has a pending invite
        let result = svc(
            Some(make_group(vec!["u1"])),
            vec!["u2"],
            vec![summary("u2")],
            true,
        )
        .get_invitable("g1", &alice())
        .await
        .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn get_invitable_returns_eligible_mutual_friends() {
        let result = svc(
            Some(make_group(vec!["u1"])),
            vec!["u2"],
            vec![summary("u2")],
            false,
        )
        .get_invitable("g1", &alice())
        .await
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].keycloak_id, "u2");
    }
}
