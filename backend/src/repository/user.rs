use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::model::user::{User, UserFullProfile, UserSummary, Visibility};
use crate::repository::traits::UserRepo;

pub struct UserRepository {
    db: Arc<Surreal<Client>>,
}

#[derive(Deserialize)]
struct FriendsRow {
    #[serde(default)]
    friends: Vec<String>,
}

impl UserRepository {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepo for UserRepository {
    /// Syncs auth fields only — never overwrites user-owned data like games.
    async fn upsert(&self, user: User) -> Result<(), surrealdb::Error> {
        let id = user.keycloak_id.clone();
        self.db.upsert::<Option<User>>(("user", id)).merge(user).await?;
        Ok(())
    }

    async fn search(&self, query: String, exclude_id: String) -> Result<Vec<UserSummary>, surrealdb::Error> {
        let mut res = self.db
            .query(
                "SELECT keycloak_id, username FROM user
                 WHERE keycloak_id != $exclude
                   AND string::contains(string::lowercase(username), string::lowercase($q))
                 LIMIT 20",
            )
            .bind(("q", query))
            .bind(("exclude", exclude_id))
            .await?;
        res.take(0)
    }

    async fn find_by_ids(&self, ids: Vec<String>) -> Result<Vec<UserSummary>, surrealdb::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let mut res = self.db
            .query("SELECT keycloak_id, username FROM user WHERE keycloak_id IN $ids")
            .bind(("ids", ids))
            .await?;
        res.take(0)
    }

    async fn add_game(&self, keycloak_id: String, game_name: String) -> Result<(), surrealdb::Error> {
        self.db
            .query("UPDATE type::thing('user', $id) SET games = array::union(games ?? [], [$game])")
            .bind(("id", keycloak_id))
            .bind(("game", game_name))
            .await?;
        Ok(())
    }

    async fn remove_game(&self, keycloak_id: String, game_name: String) -> Result<(), surrealdb::Error> {
        self.db
            .query("UPDATE type::thing('user', $id) SET games -= $game")
            .bind(("id", keycloak_id))
            .bind(("game", game_name))
            .await?;
        Ok(())
    }

    async fn get_friend_ids(&self, keycloak_id: String) -> Result<Vec<String>, surrealdb::Error> {
        let mut res = self.db
            .query("SELECT friends ?? [] as friends FROM type::thing('user', $id) LIMIT 1")
            .bind(("id", keycloak_id))
            .await?;
        let row: Option<FriendsRow> = res.take(0)?;
        Ok(row.map(|r| r.friends).unwrap_or_default())
    }

    async fn add_friend(&self, user_id: String, friend_id: String) -> Result<(), surrealdb::Error> {
        self.db
            .query(
                "UPDATE type::thing('user', $id)
                 SET friends = array::union(friends ?? [], [$friend_id])",
            )
            .bind(("id", user_id))
            .bind(("friend_id", friend_id))
            .await?;
        Ok(())
    }

    async fn remove_friend(&self, user_id: String, friend_id: String) -> Result<(), surrealdb::Error> {
        self.db
            .query("UPDATE type::thing('user', $id) SET friends -= $friend_id")
            .bind(("id", user_id))
            .bind(("friend_id", friend_id))
            .await?;
        Ok(())
    }

    async fn get_full_profile(&self, keycloak_id: String) -> Result<Option<UserFullProfile>, surrealdb::Error> {
        let mut res = self.db
            .query(
                "SELECT keycloak_id, username,
                        games ?? [] as games,
                        steam_handle,
                        steam_visibility ?? 'public' as steam_visibility,
                        discord_handle,
                        discord_visibility ?? 'public' as discord_visibility
                 FROM user WHERE keycloak_id = $id LIMIT 1",
            )
            .bind(("id", keycloak_id))
            .await?;
        res.take(0)
    }

    async fn find_who_added(&self, user_id: String, candidate_ids: Vec<String>) -> Result<Vec<UserSummary>, surrealdb::Error> {
        if candidate_ids.is_empty() {
            return Ok(vec![]);
        }
        let mut res = self.db
            .query(
                "SELECT keycloak_id, username FROM user
                 WHERE keycloak_id IN $candidates AND $user_id IN (friends ?? [])",
            )
            .bind(("candidates", candidate_ids))
            .bind(("user_id", user_id))
            .await?;
        res.take(0)
    }

    async fn update_social_profile(
        &self,
        keycloak_id: String,
        steam_handle: Option<String>,
        steam_visibility: Visibility,
        discord_handle: Option<String>,
        discord_visibility: Visibility,
    ) -> Result<(), surrealdb::Error> {
        self.db
            .query(
                "UPDATE type::thing('user', $id) SET
                     steam_handle = $steam_handle,
                     steam_visibility = $steam_visibility,
                     discord_handle = $discord_handle,
                     discord_visibility = $discord_visibility",
            )
            .bind(("id", keycloak_id))
            .bind(("steam_handle", steam_handle))
            .bind(("steam_visibility", steam_visibility))
            .bind(("discord_handle", discord_handle))
            .bind(("discord_visibility", discord_visibility))
            .await?;
        Ok(())
    }
}
