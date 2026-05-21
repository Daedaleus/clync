use std::sync::Arc;

use async_trait::async_trait;
use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::model::session_invitation::SessionInvitationRecord;
use crate::repository::traits::SessionInvitationRepo;

pub struct SessionInvitationRepository {
    db: Arc<Surreal<Client>>,
}

impl SessionInvitationRepository {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SessionInvitationRepo for SessionInvitationRepository {
    async fn create(
        &self,
        session_id: String,
        game: String,
        scheduled_at: String,
        inviter_id: String,
        inviter_username: String,
        invitee_id: String,
    ) -> Result<(), surrealdb::Error> {
        self.db
            .query(
                "CREATE session_invitation CONTENT {
                     session_id: $session_id, game: $game, scheduled_at: $scheduled_at,
                     inviter_id: $inviter_id, inviter_username: $inviter_username,
                     invitee_id: $invitee_id, created_at: time::now()
                 }",
            )
            .bind(("session_id", session_id))
            .bind(("game", game))
            .bind(("scheduled_at", scheduled_at))
            .bind(("inviter_id", inviter_id))
            .bind(("inviter_username", inviter_username))
            .bind(("invitee_id", invitee_id))
            .await?;
        Ok(())
    }

    async fn find_pending_for_user(
        &self,
        user_id: String,
    ) -> Result<Vec<SessionInvitationRecord>, surrealdb::Error> {
        let mut res = self
            .db
            .query(
                "SELECT meta::id(id) as id, session_id, game, scheduled_at,
                        inviter_id, inviter_username, invitee_id, created_at
                 FROM session_invitation WHERE invitee_id = $user_id
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
    ) -> Result<Option<SessionInvitationRecord>, surrealdb::Error> {
        let mut res = self
            .db
            .query(
                "SELECT meta::id(id) as id, session_id, game, scheduled_at,
                        inviter_id, inviter_username, invitee_id
                 FROM type::thing('session_invitation', $id)
                 WHERE invitee_id = $user_id LIMIT 1",
            )
            .bind(("id", inv_id))
            .bind(("user_id", user_id))
            .await?;
        res.take(0)
    }

    async fn exists_for_session_and_invitee(
        &self,
        session_id: String,
        invitee_id: String,
    ) -> Result<bool, surrealdb::Error> {
        let mut res = self
            .db
            .query(
                "SELECT meta::id(id) as id FROM session_invitation
                 WHERE session_id = $session_id AND invitee_id = $invitee_id LIMIT 1",
            )
            .bind(("session_id", session_id))
            .bind(("invitee_id", invitee_id))
            .await?;
        let rows: Vec<serde_json::Value> = res.take(0)?;
        Ok(!rows.is_empty())
    }

    async fn delete(&self, inv_id: String, user_id: String) -> Result<(), surrealdb::Error> {
        self.db
            .query("DELETE type::thing('session_invitation', $id) WHERE invitee_id = $user_id")
            .bind(("id", inv_id))
            .bind(("user_id", user_id))
            .await?;
        Ok(())
    }
}
