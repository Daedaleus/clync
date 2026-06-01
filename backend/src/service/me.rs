use std::sync::Arc;

use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::dto::me::{MeResponse, UpdateProfileRequest};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::repository::{
    group::GroupRepository,
    traits::{GroupRepo, UserRepo},
    user::UserRepository,
};

pub struct MeService {
    user_repo: Box<dyn UserRepo>,
    group_repo: Box<dyn GroupRepo>,
}

impl MeService {
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

    pub async fn get_me(&self, user: &AuthUser) -> Result<MeResponse, AppError> {
        let (profile, groups) = tokio::try_join!(
            self.user_repo.get_full_profile(user.keycloak_id.clone()),
            self.group_repo.find_by_member(user.keycloak_id.clone()),
        )?;

        let p = profile.unwrap_or_default();

        Ok(MeResponse {
            keycloak_id: user.keycloak_id.clone(),
            username: user.username.clone(),
            games: p.games,
            groups: groups.into_iter().map(Into::into).collect(),
            steam_handle: p.steam_handle,
            steam_visibility: p.steam_visibility,
            discord_handle: p.discord_handle,
            discord_visibility: p.discord_visibility,
        })
    }

    pub async fn update_profile(
        &self,
        keycloak_id: &str,
        req: UpdateProfileRequest,
    ) -> Result<(), AppError> {
        self.user_repo
            .update_social_profile(
                keycloak_id.to_owned(),
                req.steam_handle,
                req.steam_visibility,
                req.discord_handle,
                req.discord_visibility,
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::{
        middleware::auth::AuthUser,
        model::{
            group::{Group, MemberWithGames},
            user::{User, UserFullProfile, UserSummary, Visibility},
        },
        repository::traits::{GroupRepo, UserRepo},
    };

    // ── Fakes ─────────────────────────────────────────────────────────────────

    struct FakeMeUserRepo {
        profile: Option<UserFullProfile>,
    }

    #[async_trait]
    impl UserRepo for FakeMeUserRepo {
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
            Ok(())
        }
        async fn find_who_added(
            &self,
            _: String,
            _: Vec<String>,
        ) -> Result<Vec<UserSummary>, surrealdb::Error> {
            panic!()
        }
    }

    struct FakeMeGroupRepo {
        groups: Vec<Group>,
    }

    #[async_trait]
    impl GroupRepo for FakeMeGroupRepo {
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
            Ok(self.groups.clone())
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

    fn alice() -> AuthUser {
        AuthUser {
            keycloak_id: "alice".into(),
            username: "Alice".into(),
            is_admin: false,
        }
    }

    fn svc(profile: Option<UserFullProfile>, groups: Vec<Group>) -> MeService {
        MeService::with_repos(
            Box::new(FakeMeUserRepo { profile }),
            Box::new(FakeMeGroupRepo { groups }),
        )
    }

    // ── get_me ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_me_with_no_profile_uses_defaults() {
        let resp = svc(None, vec![]).get_me(&alice()).await.unwrap();
        assert_eq!(resp.keycloak_id, "alice");
        assert_eq!(resp.username, "Alice");
        assert!(resp.games.is_empty());
        assert!(resp.groups.is_empty());
        assert!(resp.steam_handle.is_none());
        assert!(resp.discord_handle.is_none());
    }

    #[tokio::test]
    async fn get_me_returns_profile_data() {
        let profile = UserFullProfile {
            keycloak_id: "alice".into(),
            username: "Alice".into(),
            games: vec!["CS2".into()],
            steam_handle: Some("alice_steam".into()),
            steam_visibility: Visibility::Public,
            discord_handle: Some("alice#1234".into()),
            discord_visibility: Visibility::Friends,
        };
        let resp = svc(Some(profile), vec![]).get_me(&alice()).await.unwrap();
        assert_eq!(resp.games, vec!["CS2"]);
        assert_eq!(resp.steam_handle.as_deref(), Some("alice_steam"));
        assert_eq!(resp.discord_handle.as_deref(), Some("alice#1234"));
    }

    #[tokio::test]
    async fn get_me_maps_groups() {
        let group = Group {
            id: "g1".into(),
            name: "Gamers".into(),
            is_public: true,
            members: vec![],
            creator_id: None,
            discord_invite: None,
        };
        let resp = svc(None, vec![group]).get_me(&alice()).await.unwrap();
        assert_eq!(resp.groups.len(), 1);
        assert_eq!(resp.groups[0].name, "Gamers");
    }

    // ── update_profile ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn update_profile_delegates_to_repo() {
        let req = UpdateProfileRequest {
            steam_handle: Some("new_steam".into()),
            steam_visibility: Visibility::Public,
            discord_handle: None,
            discord_visibility: Visibility::Friends,
        };
        svc(None, vec![])
            .update_profile("alice", req)
            .await
            .unwrap();
    }
}
