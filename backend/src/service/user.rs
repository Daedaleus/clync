use std::sync::Arc;

use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::dto::user::{FriendResponse, UserProfileResponse};
use crate::error::AppError;
use crate::model::user::Visibility;
use crate::repository::{
    group::GroupRepository,
    traits::{GroupRepo, UserRepo},
    user::UserRepository,
};

pub struct UserService {
    user_repo: Box<dyn UserRepo>,
    group_repo: Box<dyn GroupRepo>,
}

impl UserService {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self {
            user_repo: Box::new(UserRepository::new(Arc::clone(&db))),
            group_repo: Box::new(GroupRepository::new(db)),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_repos(user_repo: Box<dyn UserRepo>, group_repo: Box<dyn GroupRepo>) -> Self {
        Self {
            user_repo,
            group_repo,
        }
    }

    pub async fn search(
        &self,
        query: &str,
        exclude_id: &str,
    ) -> Result<Vec<FriendResponse>, AppError> {
        if query.trim().len() < 2 {
            return Ok(vec![]);
        }
        Ok(self
            .user_repo
            .search(query.to_owned(), exclude_id.to_owned())
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub async fn get_profile(
        &self,
        target_id: &str,
        requester_id: &str,
    ) -> Result<Option<UserProfileResponse>, AppError> {
        let profile = match self
            .user_repo
            .get_full_profile(target_id.to_owned())
            .await?
        {
            None => return Ok(None),
            Some(p) => p,
        };

        // Own profile always shows everything.
        if target_id == requester_id {
            return Ok(Some(UserProfileResponse {
                is_friend: false,
                keycloak_id: profile.keycloak_id,
                username: profile.username,
                games: profile.games,
                steam_handle: profile.steam_handle,
                discord_handle: profile.discord_handle,
            }));
        }

        let (requester_friends, target_friends, shares_group) = tokio::try_join!(
            self.user_repo.get_friend_ids(requester_id.to_owned()),
            self.user_repo.get_friend_ids(target_id.to_owned()),
            self.group_repo
                .share_group(requester_id.to_owned(), target_id.to_owned()),
        )?;

        // is_friend: has the requester added the target? (drives the Add/Remove button)
        let is_friend = requester_friends.contains(&target_id.to_owned());
        // Friends-visibility requires both sides to have added each other.
        let is_mutual = is_friend && target_friends.contains(&requester_id.to_owned());

        let visible = |visibility: &Visibility| -> bool {
            match visibility {
                Visibility::Public => true,
                Visibility::Group => shares_group,
                Visibility::Friends => is_mutual,
            }
        };

        Ok(Some(UserProfileResponse {
            is_friend,
            keycloak_id: profile.keycloak_id,
            username: profile.username,
            games: profile.games,
            steam_handle: visible(&profile.steam_visibility)
                .then_some(profile.steam_handle)
                .flatten(),
            discord_handle: visible(&profile.discord_visibility)
                .then_some(profile.discord_handle)
                .flatten(),
        }))
    }

    pub async fn get_friends(&self, user_id: &str) -> Result<Vec<FriendResponse>, AppError> {
        let friend_ids = self.user_repo.get_friend_ids(user_id.to_owned()).await?;
        if friend_ids.is_empty() {
            return Ok(vec![]);
        }
        Ok(self
            .user_repo
            .find_by_ids(friend_ids)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub async fn add_friend(&self, user_id: &str, friend_id: &str) -> Result<(), AppError> {
        if user_id == friend_id {
            return Err(AppError::Validation(
                "Du kannst dich nicht selbst als Freund hinzufügen".into(),
            ));
        }
        self.user_repo
            .add_friend(user_id.to_owned(), friend_id.to_owned())
            .await?;
        Ok(())
    }

    pub async fn remove_friend(&self, user_id: &str, friend_id: &str) -> Result<(), AppError> {
        self.user_repo
            .remove_friend(user_id.to_owned(), friend_id.to_owned())
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        group::{Group, MemberWithGames},
        user::{User, UserFullProfile, UserSummary},
    };
    use async_trait::async_trait;

    struct PanicUserRepo;
    struct PanicGroupRepo;

    #[async_trait]
    impl UserRepo for PanicUserRepo {
        async fn upsert(&self, _: User) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn search(&self, _: String, _: String) -> Result<Vec<UserSummary>, surrealdb::Error> {
            panic!()
        }
        async fn find_by_ids(&self, _: Vec<String>) -> Result<Vec<UserSummary>, surrealdb::Error> {
            panic!()
        }
        async fn add_game(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn remove_game(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn get_friend_ids(&self, _: String) -> Result<Vec<String>, surrealdb::Error> {
            panic!()
        }
        async fn add_friend(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn remove_friend(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn get_full_profile(
            &self,
            _: String,
        ) -> Result<Option<UserFullProfile>, surrealdb::Error> {
            panic!()
        }
        async fn update_social_profile(
            &self,
            _: String,
            _: Option<String>,
            _: Visibility,
            _: Option<String>,
            _: Visibility,
        ) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn find_who_added(
            &self,
            _: String,
            _: Vec<String>,
        ) -> Result<Vec<UserSummary>, surrealdb::Error> {
            panic!()
        }
    }

    #[async_trait]
    impl GroupRepo for PanicGroupRepo {
        async fn create(
            &self,
            _: String,
            _: bool,
            _: String,
        ) -> Result<Option<Group>, surrealdb::Error> {
            panic!()
        }
        async fn find_by_id(&self, _: String) -> Result<Option<Group>, surrealdb::Error> {
            panic!()
        }
        async fn find_member_games(
            &self,
            _: Vec<String>,
        ) -> Result<Vec<MemberWithGames>, surrealdb::Error> {
            panic!()
        }
        async fn list_public(&self) -> Result<Vec<Group>, surrealdb::Error> {
            panic!()
        }
        async fn search_public(&self, _: String) -> Result<Vec<Group>, surrealdb::Error> {
            panic!()
        }
        async fn find_by_member(&self, _: String) -> Result<Vec<Group>, surrealdb::Error> {
            panic!()
        }
        async fn find_public_by_member(&self, _: String) -> Result<Vec<Group>, surrealdb::Error> {
            panic!()
        }
        async fn join(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn share_group(&self, _: String, _: String) -> Result<bool, surrealdb::Error> {
            panic!()
        }
        async fn delete(&self, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn find_all(&self) -> Result<Vec<Group>, surrealdb::Error> {
            panic!()
        }
        async fn set_discord_invite(
            &self,
            _: String,
            _: Option<String>,
        ) -> Result<(), surrealdb::Error> {
            panic!()
        }
    }

    fn svc() -> UserService {
        UserService::with_repos(Box::new(PanicUserRepo), Box::new(PanicGroupRepo))
    }

    #[tokio::test]
    async fn add_friend_self_returns_error() {
        let err = svc().add_friend("user1", "user1").await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn search_one_char_returns_empty_without_db_call() {
        let result = svc().search("a", "someone").await;
        assert!(matches!(result, Ok(v) if v.is_empty()));
    }

    #[tokio::test]
    async fn search_empty_returns_empty_without_db_call() {
        let result = svc().search("", "someone").await;
        assert!(matches!(result, Ok(v) if v.is_empty()));
    }
}
