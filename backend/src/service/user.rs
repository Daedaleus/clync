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
    async fn search_one_char_returns_empty_without_db_call() {
        let result = svc().search("a", "someone").await;
        assert!(matches!(result, Ok(v) if v.is_empty()));
    }

    #[tokio::test]
    async fn search_empty_returns_empty_without_db_call() {
        let result = svc().search("", "someone").await;
        assert!(matches!(result, Ok(v) if v.is_empty()));
    }

    // ── get_profile — Fake repos ───────────────────────────────────────────────

    struct FakeProfileUserRepo {
        profile: Option<UserFullProfile>,
        friends: std::collections::HashMap<String, Vec<String>>,
    }

    impl FakeProfileUserRepo {
        fn with_profile(p: UserFullProfile) -> Self {
            Self {
                profile: Some(p),
                friends: std::collections::HashMap::new(),
            }
        }
        fn add_friends(mut self, user: &str, ids: &[&str]) -> Self {
            self.friends
                .insert(user.to_owned(), ids.iter().map(|s| s.to_string()).collect());
            self
        }
    }

    #[async_trait]
    impl UserRepo for FakeProfileUserRepo {
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
        async fn get_friend_ids(&self, id: String) -> Result<Vec<String>, surrealdb::Error> {
            Ok(self.friends.get(&id).cloned().unwrap_or_default())
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
            Ok(self.profile.clone())
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

    struct FakeShareGroupRepo {
        shares: bool,
    }

    #[async_trait]
    impl GroupRepo for FakeShareGroupRepo {
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
            Ok(self.shares)
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

    fn profile_with_vis(
        id: &str,
        steam_vis: Visibility,
        discord_vis: Visibility,
    ) -> UserFullProfile {
        UserFullProfile {
            keycloak_id: id.to_owned(),
            username: format!("user_{id}"),
            games: vec![],
            steam_handle: Some("steam_h".to_owned()),
            steam_visibility: steam_vis,
            discord_handle: Some("disc_h".to_owned()),
            discord_visibility: discord_vis,
        }
    }

    fn profile_svc(user_repo: FakeProfileUserRepo, shares_group: bool) -> UserService {
        UserService::with_repos(
            Box::new(user_repo),
            Box::new(FakeShareGroupRepo {
                shares: shares_group,
            }),
        )
    }

    // ── get_profile tests ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_profile_returns_none_for_unknown_user() {
        let svc = UserService::with_repos(
            Box::new(FakeProfileUserRepo {
                profile: None,
                friends: Default::default(),
            }),
            Box::new(FakeShareGroupRepo { shares: false }),
        );
        assert!(
            svc.get_profile("unknown", "requester")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn get_profile_own_profile_shows_all_fields_regardless_of_visibility() {
        let svc = profile_svc(
            FakeProfileUserRepo::with_profile(profile_with_vis(
                "alice",
                Visibility::Friends,
                Visibility::Friends,
            )),
            false,
        );
        let resp = svc.get_profile("alice", "alice").await.unwrap().unwrap();
        assert!(resp.steam_handle.is_some());
        assert!(resp.discord_handle.is_some());
        assert!(
            !resp.is_friend,
            "own profile should not mark self as friend"
        );
    }

    #[tokio::test]
    async fn get_profile_public_handles_visible_to_stranger() {
        let svc = profile_svc(
            FakeProfileUserRepo::with_profile(profile_with_vis(
                "bob",
                Visibility::Public,
                Visibility::Public,
            )),
            false,
        );
        let resp = svc.get_profile("bob", "alice").await.unwrap().unwrap();
        assert!(resp.steam_handle.is_some());
        assert!(resp.discord_handle.is_some());
    }

    #[tokio::test]
    async fn get_profile_friends_only_handles_hidden_from_stranger() {
        let svc = profile_svc(
            FakeProfileUserRepo::with_profile(profile_with_vis(
                "bob",
                Visibility::Friends,
                Visibility::Friends,
            )),
            false,
        );
        let resp = svc.get_profile("bob", "alice").await.unwrap().unwrap();
        assert!(resp.steam_handle.is_none());
        assert!(resp.discord_handle.is_none());
    }

    #[tokio::test]
    async fn get_profile_friends_only_handles_visible_when_mutual() {
        // alice added bob AND bob added alice back
        let svc = profile_svc(
            FakeProfileUserRepo::with_profile(profile_with_vis(
                "bob",
                Visibility::Friends,
                Visibility::Friends,
            ))
            .add_friends("alice", &["bob"])
            .add_friends("bob", &["alice"]),
            false,
        );
        let resp = svc.get_profile("bob", "alice").await.unwrap().unwrap();
        assert!(resp.steam_handle.is_some());
        assert!(resp.discord_handle.is_some());
        assert!(resp.is_friend);
    }

    #[tokio::test]
    async fn get_profile_friends_only_handles_hidden_when_one_way_friend() {
        // alice added bob, but bob did NOT add alice back
        let svc = profile_svc(
            FakeProfileUserRepo::with_profile(profile_with_vis(
                "bob",
                Visibility::Friends,
                Visibility::Friends,
            ))
            .add_friends("alice", &["bob"]),
            false,
        );
        let resp = svc.get_profile("bob", "alice").await.unwrap().unwrap();
        assert!(resp.steam_handle.is_none(), "not mutual → Friends hidden");
        assert!(resp.discord_handle.is_none());
        assert!(resp.is_friend, "alice added bob, so button shows Remove");
    }

    #[tokio::test]
    async fn get_profile_group_visibility_shown_to_group_member() {
        let svc = profile_svc(
            FakeProfileUserRepo::with_profile(profile_with_vis(
                "bob",
                Visibility::Group,
                Visibility::Group,
            )),
            true, // shares a group
        );
        let resp = svc.get_profile("bob", "alice").await.unwrap().unwrap();
        assert!(resp.steam_handle.is_some());
        assert!(resp.discord_handle.is_some());
    }

    #[tokio::test]
    async fn get_profile_group_visibility_hidden_without_shared_group() {
        let svc = profile_svc(
            FakeProfileUserRepo::with_profile(profile_with_vis(
                "bob",
                Visibility::Group,
                Visibility::Group,
            )),
            false,
        );
        let resp = svc.get_profile("bob", "alice").await.unwrap().unwrap();
        assert!(resp.steam_handle.is_none());
        assert!(resp.discord_handle.is_none());
    }

    #[tokio::test]
    async fn get_profile_is_friend_true_when_requester_added_target() {
        let svc = profile_svc(
            FakeProfileUserRepo::with_profile(profile_with_vis(
                "bob",
                Visibility::Public,
                Visibility::Public,
            ))
            .add_friends("alice", &["bob"]),
            false,
        );
        let resp = svc.get_profile("bob", "alice").await.unwrap().unwrap();
        assert!(resp.is_friend);
    }

    #[tokio::test]
    async fn get_profile_is_friend_false_when_not_added() {
        let svc = profile_svc(
            FakeProfileUserRepo::with_profile(profile_with_vis(
                "bob",
                Visibility::Public,
                Visibility::Public,
            )),
            false,
        );
        let resp = svc.get_profile("bob", "alice").await.unwrap().unwrap();
        assert!(!resp.is_friend);
    }

    // ── get_friends / search / remove_friend — fakes ───────────────────────────

    struct FriendsUserRepo {
        friend_ids: Vec<String>,
        by_ids_result: Vec<UserSummary>,
        search_result: Vec<UserSummary>,
    }

    impl FriendsUserRepo {
        fn empty() -> Self {
            Self {
                friend_ids: vec![],
                by_ids_result: vec![],
                search_result: vec![],
            }
        }
        fn with_friends(ids: &[&str], summaries: &[(&str, &str)]) -> Self {
            Self {
                friend_ids: ids.iter().map(|s| s.to_string()).collect(),
                by_ids_result: summaries
                    .iter()
                    .map(|(id, name)| UserSummary {
                        keycloak_id: id.to_string(),
                        username: name.to_string(),
                    })
                    .collect(),
                search_result: vec![],
            }
        }
        fn with_search(results: &[(&str, &str)]) -> Self {
            Self {
                friend_ids: vec![],
                by_ids_result: vec![],
                search_result: results
                    .iter()
                    .map(|(id, name)| UserSummary {
                        keycloak_id: id.to_string(),
                        username: name.to_string(),
                    })
                    .collect(),
            }
        }
    }

    #[async_trait]
    impl UserRepo for FriendsUserRepo {
        async fn upsert(&self, _: User) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn search(&self, _: String, _: String) -> Result<Vec<UserSummary>, surrealdb::Error> {
            Ok(self.search_result.clone())
        }
        async fn find_by_ids(&self, _: Vec<String>) -> Result<Vec<UserSummary>, surrealdb::Error> {
            Ok(self.by_ids_result.clone())
        }
        async fn add_game(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn remove_game(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn get_friend_ids(&self, _: String) -> Result<Vec<String>, surrealdb::Error> {
            Ok(self.friend_ids.clone())
        }
        async fn add_friend(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn remove_friend(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            Ok(())
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

    // ── get_friends ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_friends_empty_returns_empty_without_db_lookup() {
        let svc =
            UserService::with_repos(Box::new(FriendsUserRepo::empty()), Box::new(PanicGroupRepo));
        let result = svc.get_friends("alice").await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn get_friends_returns_mapped_results() {
        let svc = UserService::with_repos(
            Box::new(FriendsUserRepo::with_friends(&["bob"], &[("bob", "Bob")])),
            Box::new(PanicGroupRepo),
        );
        let result = svc.get_friends("alice").await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].keycloak_id, "bob");
    }

    // ── search ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn search_with_sufficient_query_returns_repo_results() {
        let svc = UserService::with_repos(
            Box::new(FriendsUserRepo::with_search(&[("bob", "Bob")])),
            Box::new(PanicGroupRepo),
        );
        let result = svc.search("bo", "alice").await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].keycloak_id, "bob");
    }

    // ── remove_friend ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn remove_friend_delegates_to_repo() {
        let svc =
            UserService::with_repos(Box::new(FriendsUserRepo::empty()), Box::new(PanicGroupRepo));
        svc.remove_friend("alice", "bob").await.unwrap();
    }
}
