use std::sync::Arc;

use async_trait::async_trait;
use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::model::friend_request::FriendRequest;
use crate::repository::traits::FriendRequestRepo;

pub struct FriendRequestRepository {
    db: Arc<Surreal<Client>>,
}

impl FriendRequestRepository {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl FriendRequestRepo for FriendRequestRepository {
    async fn create(
        &self,
        from_id: String,
        from_username: String,
        to_id: String,
    ) -> Result<(), surrealdb::Error> {
        self.db
            .query(
                "CREATE friend_request CONTENT {
                    from_id: $from_id, from_username: $from_username, to_id: $to_id
                }",
            )
            .bind(("from_id", from_id))
            .bind(("from_username", from_username))
            .bind(("to_id", to_id))
            .await?;
        Ok(())
    }

    async fn find_incoming(&self, user_id: String) -> Result<Vec<FriendRequest>, surrealdb::Error> {
        let mut res = self
            .db
            .query(
                "SELECT meta::id(id) as id, from_id, from_username, to_id
                 FROM friend_request WHERE to_id = $user_id",
            )
            .bind(("user_id", user_id))
            .await?;
        res.take(0)
    }

    async fn find_by_id_for_receiver(
        &self,
        req_id: String,
        receiver_id: String,
    ) -> Result<Option<FriendRequest>, surrealdb::Error> {
        let mut res = self
            .db
            .query(
                "SELECT meta::id(id) as id, from_id, from_username, to_id
                 FROM type::thing('friend_request', $id)
                 WHERE to_id = $receiver_id LIMIT 1",
            )
            .bind(("id", req_id))
            .bind(("receiver_id", receiver_id))
            .await?;
        res.take(0)
    }

    async fn exists_between(
        &self,
        user_a: String,
        user_b: String,
    ) -> Result<bool, surrealdb::Error> {
        let mut res = self
            .db
            .query(
                "SELECT meta::id(id) as id FROM friend_request
                 WHERE (from_id = $a AND to_id = $b) OR (from_id = $b AND to_id = $a)
                 LIMIT 1",
            )
            .bind(("a", user_a))
            .bind(("b", user_b))
            .await?;
        let rows: Vec<serde_json::Value> = res.take(0)?;
        Ok(!rows.is_empty())
    }

    async fn delete(&self, req_id: String) -> Result<(), surrealdb::Error> {
        self.db
            .query("DELETE type::thing('friend_request', $id)")
            .bind(("id", req_id))
            .await?;
        Ok(())
    }

    async fn delete_for_participant(
        &self,
        req_id: String,
        user_id: String,
    ) -> Result<bool, surrealdb::Error> {
        let mut res = self
            .db
            .query(
                "DELETE type::thing('friend_request', $id)
                 WHERE from_id = $user OR to_id = $user
                 RETURN BEFORE",
            )
            .bind(("id", req_id))
            .bind(("user", user_id))
            .await?;
        let rows: Vec<serde_json::Value> = res.take(0)?;
        Ok(!rows.is_empty())
    }
}
