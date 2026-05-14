use std::sync::Arc;

use async_trait::async_trait;
use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::model::group::{Group, MemberWithGames};
use crate::repository::traits::GroupRepo;

pub struct GroupRepository {
    db: Arc<Surreal<Client>>,
}

impl GroupRepository {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl GroupRepo for GroupRepository {
    async fn create(
        &self,
        name: String,
        is_public: bool,
        creator_id: String,
    ) -> Result<Option<Group>, surrealdb::Error> {
        let mut res = self.db
            .query(
                "CREATE group CONTENT { name: $name, is_public: $is_public, members: [$creator_id] }
                 RETURN meta::id(id) as id, name, is_public, members",
            )
            .bind(("name", name))
            .bind(("is_public", is_public))
            .bind(("creator_id", creator_id))
            .await?;
        res.take(0)
    }

    async fn find_by_id(&self, group_id: String) -> Result<Option<Group>, surrealdb::Error> {
        let mut res = self
            .db
            .query(
                "SELECT meta::id(id) as id, name, is_public, members
                 FROM type::thing('group', $id)",
            )
            .bind(("id", group_id))
            .await?;
        res.take(0)
    }

    async fn find_member_games(
        &self,
        member_ids: Vec<String>,
    ) -> Result<Vec<MemberWithGames>, surrealdb::Error> {
        let mut res = self
            .db
            .query(
                "SELECT keycloak_id, username, games ?? [] as games
                 FROM user WHERE keycloak_id IN $members",
            )
            .bind(("members", member_ids))
            .await?;
        res.take(0)
    }

    async fn list_public(&self) -> Result<Vec<Group>, surrealdb::Error> {
        let mut res = self
            .db
            .query(
                "SELECT meta::id(id) as id, name, is_public, members
                 FROM group WHERE is_public = true LIMIT 50",
            )
            .await?;
        res.take(0)
    }

    async fn search_public(&self, query: String) -> Result<Vec<Group>, surrealdb::Error> {
        let mut res = self
            .db
            .query(
                "SELECT meta::id(id) as id, name, is_public, members
                 FROM group
                 WHERE is_public = true
                   AND string::contains(string::lowercase(name), string::lowercase($q))
                 LIMIT 20",
            )
            .bind(("q", query))
            .await?;
        res.take(0)
    }

    async fn find_by_member(&self, keycloak_id: String) -> Result<Vec<Group>, surrealdb::Error> {
        let mut res = self
            .db
            .query(
                "SELECT meta::id(id) as id, name, is_public, members
                 FROM group WHERE members CONTAINS $user_id",
            )
            .bind(("user_id", keycloak_id))
            .await?;
        res.take(0)
    }

    /// Uses array::union to prevent duplicate members.
    async fn join(&self, group_id: String, keycloak_id: String) -> Result<(), surrealdb::Error> {
        self.db
            .query(
                "UPDATE type::thing('group', $id)
                 SET members = array::union(members, [$user_id])",
            )
            .bind(("id", group_id))
            .bind(("user_id", keycloak_id))
            .await?;
        Ok(())
    }

    async fn share_group(&self, user_a: String, user_b: String) -> Result<bool, surrealdb::Error> {
        let mut res = self.db
            .query("SELECT meta::id(id) as id FROM group WHERE members CONTAINS $a AND members CONTAINS $b LIMIT 1")
            .bind(("a", user_a))
            .bind(("b", user_b))
            .await?;
        let rows: Vec<serde_json::Value> = res.take(0)?;
        Ok(!rows.is_empty())
    }

    async fn delete(&self, group_id: String) -> Result<(), surrealdb::Error> {
        self.db
            .query("DELETE type::thing('group', $id)")
            .bind(("id", group_id))
            .await?;
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Group>, surrealdb::Error> {
        let mut res = self
            .db
            .query("SELECT meta::id(id) as id, name, is_public, members FROM group ORDER BY name")
            .await?;
        res.take(0)
    }
}
