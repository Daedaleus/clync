use std::sync::Arc;

use async_trait::async_trait;
use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::model::invitation::InvitationRecord;
use crate::repository::traits::InvitationRepo;

pub struct InvitationRepository {
    db: Arc<Surreal<Client>>,
}

impl InvitationRepository {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl InvitationRepo for InvitationRepository {
    async fn create(
        &self,
        group_id: String,
        group_name: String,
        inviter_id: String,
        inviter_username: String,
        invitee_id: String,
    ) -> Result<(), surrealdb::Error> {
        self.db
            .query(
                "CREATE invitation CONTENT {
                     group_id: $group_id, group_name: $group_name,
                     inviter_id: $inviter_id, inviter_username: $inviter_username,
                     invitee_id: $invitee_id, created_at: time::now()
                 }",
            )
            .bind(("group_id", group_id))
            .bind(("group_name", group_name))
            .bind(("inviter_id", inviter_id))
            .bind(("inviter_username", inviter_username))
            .bind(("invitee_id", invitee_id))
            .await?;
        Ok(())
    }

    async fn find_pending_for_user(
        &self,
        user_id: String,
    ) -> Result<Vec<InvitationRecord>, surrealdb::Error> {
        let mut res = self
            .db
            .query(
                "SELECT meta::id(id) as id, group_id, group_name,
                        inviter_id, inviter_username, invitee_id, created_at
                 FROM invitation WHERE invitee_id = $user_id
                 ORDER BY created_at DESC",
            )
            .bind(("user_id", user_id))
            .await?;
        res.take(0)
    }

    async fn find_by_id_for_invitee(
        &self,
        inv_id: String,
        user_id: String,
    ) -> Result<Option<InvitationRecord>, surrealdb::Error> {
        let mut res = self.db
            .query(
                "SELECT meta::id(id) as id, group_id, group_name, inviter_id, inviter_username, invitee_id
                 FROM type::thing('invitation', $id) WHERE invitee_id = $user_id LIMIT 1",
            )
            .bind(("id", inv_id))
            .bind(("user_id", user_id))
            .await?;
        res.take(0)
    }

    async fn exists_for_group_and_invitee(
        &self,
        group_id: String,
        invitee_id: String,
    ) -> Result<bool, surrealdb::Error> {
        let mut res = self.db
            .query("SELECT meta::id(id) as id FROM invitation WHERE group_id = $group_id AND invitee_id = $invitee_id LIMIT 1")
            .bind(("group_id", group_id))
            .bind(("invitee_id", invitee_id))
            .await?;
        let rows: Vec<serde_json::Value> = res.take(0)?;
        Ok(!rows.is_empty())
    }

    async fn delete(&self, inv_id: String, user_id: String) -> Result<(), surrealdb::Error> {
        self.db
            .query("DELETE type::thing('invitation', $id) WHERE invitee_id = $user_id")
            .bind(("id", inv_id))
            .bind(("user_id", user_id))
            .await?;
        Ok(())
    }
}
