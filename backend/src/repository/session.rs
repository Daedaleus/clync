use std::sync::Arc;

use async_trait::async_trait;
use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::model::session::{Session, SessionDetail};
use crate::repository::traits::SessionRepo;

pub struct SessionRepository {
    db: Arc<Surreal<Client>>,
}

const SELECT_FIELDS: &str =
    "meta::id(id) as id, user_id, username, game, scheduled_at, scope, group_ids,
     participants ?? [] as participants,
     (SELECT VALUE name FROM group WHERE meta::id(id) IN $parent.group_ids) as group_names,
     (SELECT VALUE (thumbnail_b64 IS NOT NONE) FROM type::thing('game', $parent.game) LIMIT 1)[0] ?? false as game_has_thumbnail";

impl SessionRepository {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SessionRepo for SessionRepository {
    async fn create(
        &self,
        user_id: String,
        username: String,
        game: String,
        scheduled_at: String,
        scope: String,
        group_ids: Vec<String>,
    ) -> Result<Option<Session>, surrealdb::Error> {
        let mut res = self
            .db
            .query(format!(
                "CREATE session CONTENT {{
                    user_id: $user_id, username: $username, game: $game,
                    scheduled_at: $scheduled_at, scope: $scope,
                    group_ids: $group_ids, participants: []
                 }} RETURN {SELECT_FIELDS}"
            ))
            .bind(("user_id", user_id))
            .bind(("username", username))
            .bind(("game", game))
            .bind(("scheduled_at", scheduled_at))
            .bind(("scope", scope))
            .bind(("group_ids", group_ids))
            .await?;
        res.take(0)
    }

    async fn find_by_id(
        &self,
        session_id: String,
    ) -> Result<Option<SessionDetail>, surrealdb::Error> {
        let mut res = self.db
            .query(
                "SELECT meta::id(id) as id, user_id, username, game, scheduled_at, scope, group_ids,
                        (SELECT keycloak_id, username FROM user
                         WHERE keycloak_id IN $parent.participants ?? []) as participants,
                        (SELECT VALUE name FROM group WHERE meta::id(id) IN $parent.group_ids) as group_names,
                        (SELECT VALUE (thumbnail_b64 IS NOT NONE) FROM type::thing('game', $parent.game) LIMIT 1)[0] ?? false as game_has_thumbnail
                 FROM type::thing('session', $id) LIMIT 1",
            )
            .bind(("id", session_id))
            .await?;
        res.take(0)
    }

    async fn find_feed(&self, my_group_ids: Vec<String>) -> Result<Vec<Session>, surrealdb::Error> {
        let mut res = self
            .db
            .query(format!(
                "SELECT {SELECT_FIELDS} FROM session
                 WHERE scheduled_at >= time::now() - 2h
                   AND (scope = 'global'
                    OR (scope = 'groups' AND group_ids CONTAINSANY $group_ids))
                 ORDER BY scheduled_at ASC LIMIT 100"
            ))
            .bind(("group_ids", my_group_ids))
            .await?;
        res.take(0)
    }

    async fn find_mine(&self, user_id: String) -> Result<Vec<Session>, surrealdb::Error> {
        let mut res = self
            .db
            .query(format!(
                "SELECT {SELECT_FIELDS} FROM session
                 WHERE scheduled_at >= time::now() - 2h
                   AND (user_id = $user_id OR participants CONTAINS $user_id)
                 ORDER BY scheduled_at ASC"
            ))
            .bind(("user_id", user_id))
            .await?;
        res.take(0)
    }

    async fn find_for_group(&self, group_id: String) -> Result<Vec<Session>, surrealdb::Error> {
        let mut res = self
            .db
            .query(format!(
                "SELECT {SELECT_FIELDS} FROM session
                 WHERE scheduled_at >= time::now() - 2h
                   AND scope = 'groups' AND group_ids CONTAINS $group_id
                 ORDER BY scheduled_at ASC"
            ))
            .bind(("group_id", group_id))
            .await?;
        res.take(0)
    }

    async fn join(
        &self,
        session_id: String,
        user_id: String,
    ) -> Result<Option<Session>, surrealdb::Error> {
        let mut res = self
            .db
            .query(format!(
                "UPDATE type::thing('session', $id)
                 SET participants = array::union(participants ?? [], [$user_id])
                 WHERE user_id != $user_id
                 RETURN {SELECT_FIELDS}"
            ))
            .bind(("id", session_id))
            .bind(("user_id", user_id))
            .await?;
        res.take(0)
    }

    async fn leave(&self, session_id: String, user_id: String) -> Result<(), surrealdb::Error> {
        self.db
            .query("UPDATE type::thing('session', $id) SET participants -= $user_id WHERE user_id != $user_id")
            .bind(("id", session_id))
            .bind(("user_id", user_id))
            .await?;
        Ok(())
    }

    async fn delete_as_admin(&self, session_id: String) -> Result<Vec<String>, surrealdb::Error> {
        #[derive(serde::Deserialize)]
        struct SessionBefore {
            #[serde(default)]
            group_ids: Vec<String>,
        }
        let mut res = self
            .db
            .query("DELETE type::thing('session', $id) RETURN BEFORE")
            .bind(("id", session_id))
            .await?;
        let row: Option<SessionBefore> = res.take(0)?;
        Ok(row.map(|r| r.group_ids).unwrap_or_default())
    }

    async fn delete(
        &self,
        session_id: String,
        user_id: String,
    ) -> Result<Vec<String>, surrealdb::Error> {
        #[derive(serde::Deserialize)]
        struct SessionBefore {
            #[serde(default)]
            group_ids: Vec<String>,
        }

        let mut res = self
            .db
            .query("DELETE type::thing('session', $id) WHERE user_id = $user_id RETURN BEFORE")
            .bind(("id", session_id))
            .bind(("user_id", user_id))
            .await?;
        let row: Option<SessionBefore> = res.take(0)?;
        Ok(row.map(|r| r.group_ids).unwrap_or_default())
    }
}
