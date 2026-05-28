use std::sync::Arc;

use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::dto::group::{
    CreateGroupRequest, GroupDetailResponse, GroupResponse, GroupSummary, MemberResponse,
    PossibleGame,
};
use crate::error::{AppError, codes};
use crate::model::group::MemberWithGames;
use crate::repository::{
    group::GroupRepository,
    traits::{GroupRepo, UserRepo},
    user::UserRepository,
};

pub struct GroupService {
    repo: Box<dyn GroupRepo>,
    user_repo: Box<dyn UserRepo>,
}

impl GroupService {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self {
            repo: Box::new(GroupRepository::new(Arc::clone(&db))),
            user_repo: Box::new(UserRepository::new(db)),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_repos(repo: Box<dyn GroupRepo>, user_repo: Box<dyn UserRepo>) -> Self {
        Self { repo, user_repo }
    }

    pub async fn create(
        &self,
        req: CreateGroupRequest,
        creator_id: &str,
    ) -> Result<GroupResponse, AppError> {
        let name = req.name.trim().to_owned();
        if name.is_empty() {
            return Err(AppError::Validation(codes::group::NAME_REQUIRED.into()));
        }
        let group = self
            .repo
            .create(name, req.is_public, creator_id.to_owned())
            .await?
            .ok_or_else(|| AppError::Internal("Group creation returned no result".into()))?;
        Ok(group.into())
    }

    pub async fn get_detail(
        &self,
        group_id: &str,
        requester_id: &str,
    ) -> Result<Option<GroupDetailResponse>, AppError> {
        let group = match self.repo.find_by_id(group_id.to_owned()).await? {
            None => return Ok(None),
            Some(g) => g,
        };

        let (members_with_games, friend_ids) = tokio::try_join!(
            async {
                if group.members.is_empty() {
                    Ok(vec![])
                } else {
                    self.repo.find_member_games(group.members).await
                }
            },
            self.user_repo.get_friend_ids(requester_id.to_owned()),
        )?;

        let common_games = intersect_games(&members_with_games);
        let possible = possible_games(&members_with_games);
        let members = members_with_games
            .into_iter()
            .map(|m| {
                let is_friend = friend_ids.contains(&m.keycloak_id);
                MemberResponse {
                    keycloak_id: m.keycloak_id,
                    username: m.username,
                    is_friend,
                }
            })
            .collect();

        Ok(Some(GroupDetailResponse {
            id: group.id,
            name: group.name,
            is_public: group.is_public,
            creator_id: group.creator_id,
            discord_invite: group.discord_invite,
            members,
            common_games,
            possible_games: possible,
        }))
    }

    pub async fn search_public(
        &self,
        query: &str,
        is_admin: bool,
    ) -> Result<Vec<GroupResponse>, AppError> {
        let groups = if is_admin {
            self.repo.find_all().await?
        } else if query.is_empty() {
            self.repo.list_public().await?
        } else {
            self.repo.search_public(query.to_owned()).await?
        };
        Ok(groups.into_iter().map(Into::into).collect())
    }

    pub async fn my_groups(&self, keycloak_id: &str) -> Result<Vec<GroupSummary>, AppError> {
        Ok(self
            .repo
            .find_by_member(keycloak_id.to_owned())
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub async fn join(&self, group_id: &str, keycloak_id: &str) -> Result<(), AppError> {
        self.repo
            .join(group_id.to_owned(), keycloak_id.to_owned())
            .await?;
        Ok(())
    }

    pub async fn delete(&self, group_id: &str) -> Result<(), AppError> {
        self.repo.delete(group_id.to_owned()).await?;
        Ok(())
    }

    pub async fn set_discord_invite(
        &self,
        group_id: &str,
        url: Option<String>,
        requester_id: &str,
        is_admin: bool,
    ) -> Result<(), AppError> {
        // Validate URL format when provided
        if let Some(ref u) = url {
            let valid = u.starts_with("https://discord.gg/")
                || u.starts_with("https://discord.com/invite/");
            if !valid {
                return Err(AppError::Validation(
                    codes::group::INVALID_DISCORD_URL.into(),
                ));
            }
            if u.len() > 200 {
                return Err(AppError::Validation(
                    codes::group::DISCORD_URL_TOO_LONG.into(),
                ));
            }
        }

        let group = self
            .repo
            .find_by_id(group_id.to_owned())
            .await?
            .ok_or_else(|| AppError::Validation(codes::group::NOT_FOUND.into()))?;

        let is_creator = group.creator_id.as_deref() == Some(requester_id);
        if !is_creator && !is_admin {
            return Err(AppError::Unauthorized(
                codes::group::DISCORD_URL_UNAUTHORIZED.into(),
            ));
        }

        self.repo
            .set_discord_invite(group_id.to_owned(), url)
            .await?;
        Ok(())
    }
}

/// Returns all games any member has, sorted by count desc then name asc.
pub(crate) fn possible_games(members: &[MemberWithGames]) -> Vec<PossibleGame> {
    use std::collections::HashMap;
    let total = members.len();
    let mut counts: HashMap<String, usize> = HashMap::new();
    for member in members {
        for game in &member.games {
            *counts.entry(game.clone()).or_insert(0) += 1;
        }
    }
    let mut games: Vec<PossibleGame> = counts
        .into_iter()
        .map(|(name, count)| PossibleGame { name, count, total })
        .collect();
    games.sort_by(|a, b| b.count.cmp(&a.count).then(a.name.cmp(&b.name)));
    games
}

/// Returns the sorted intersection of all members' game lists.
pub(crate) fn intersect_games(members: &[MemberWithGames]) -> Vec<String> {
    if members.is_empty() {
        return vec![];
    }
    let mut result = members[0].games.clone();
    for member in members.iter().skip(1) {
        result.retain(|game| member.games.contains(game));
    }
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::group::{Group, MemberWithGames};
    use async_trait::async_trait;

    fn member(username: &str, games: &[&str]) -> MemberWithGames {
        MemberWithGames {
            keycloak_id: format!("id_{username}"),
            username: username.to_owned(),
            games: games.iter().map(|s| s.to_string()).collect(),
        }
    }

    // ── intersect_games ────────────────────────────────────────────────────────

    #[test]
    fn intersect_empty_returns_empty() {
        assert_eq!(intersect_games(&[]), Vec::<String>::new());
    }

    #[test]
    fn intersect_single_member_returns_their_games() {
        let result = intersect_games(&[member("alice", &["CS2", "Minecraft"])]);
        assert_eq!(result, vec!["CS2", "Minecraft"]);
    }

    #[test]
    fn intersect_two_members_common_subset() {
        let members = vec![
            member("alice", &["CS2", "Minecraft", "LoL"]),
            member("bob", &["CS2", "Dota2", "LoL"]),
        ];
        assert_eq!(intersect_games(&members), vec!["CS2", "LoL"]);
    }

    #[test]
    fn intersect_no_common_games_returns_empty() {
        let members = vec![member("alice", &["CS2"]), member("bob", &["Minecraft"])];
        assert_eq!(intersect_games(&members), Vec::<String>::new());
    }

    #[test]
    fn intersect_result_is_sorted_alphabetically() {
        let members = vec![
            member("alice", &["Zelda", "CS2", "Minecraft"]),
            member("bob", &["CS2", "Zelda", "Minecraft"]),
        ];
        assert_eq!(intersect_games(&members), vec!["CS2", "Minecraft", "Zelda"]);
    }

    #[test]
    fn intersect_three_members_only_all_common() {
        let members = vec![
            member("alice", &["CS2", "LoL", "Minecraft"]),
            member("bob", &["CS2", "LoL", "Dota2"]),
            member("carol", &["CS2", "Apex", "Minecraft"]),
        ];
        assert_eq!(intersect_games(&members), vec!["CS2"]);
    }

    // ── GroupService::create validation ────────────────────────────────────────

    struct PanicGroupRepo;
    struct PanicUserRepo;

    #[async_trait]
    impl GroupRepo for PanicGroupRepo {
        async fn create(
            &self,
            _: String,
            _: bool,
            _: String,
        ) -> Result<Option<Group>, surrealdb::Error> {
            panic!("should not be called")
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

    #[async_trait]
    impl UserRepo for PanicUserRepo {
        async fn upsert(&self, _: crate::model::user::User) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn search(
            &self,
            _: String,
            _: String,
        ) -> Result<Vec<crate::model::user::UserSummary>, surrealdb::Error> {
            panic!()
        }
        async fn find_by_ids(
            &self,
            _: Vec<String>,
        ) -> Result<Vec<crate::model::user::UserSummary>, surrealdb::Error> {
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
        ) -> Result<Option<crate::model::user::UserFullProfile>, surrealdb::Error> {
            panic!()
        }
        async fn update_social_profile(
            &self,
            _: String,
            _: Option<String>,
            _: crate::model::user::Visibility,
            _: Option<String>,
            _: crate::model::user::Visibility,
        ) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn find_who_added(
            &self,
            _: String,
            _: Vec<String>,
        ) -> Result<Vec<crate::model::user::UserSummary>, surrealdb::Error> {
            panic!()
        }
    }

    fn svc() -> GroupService {
        GroupService::with_repos(Box::new(PanicGroupRepo), Box::new(PanicUserRepo))
    }

    #[tokio::test]
    async fn create_empty_name_returns_error() {
        let req = CreateGroupRequest {
            name: "   ".into(),
            is_public: true,
        };
        assert!(svc().create(req, "user1").await.is_err());
    }

    #[tokio::test]
    async fn create_empty_string_returns_error() {
        let req = CreateGroupRequest {
            name: "".into(),
            is_public: true,
        };
        assert!(svc().create(req, "user1").await.is_err());
    }
}
