use std::sync::Arc;

use async_trait::async_trait;
use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::model::invite::InviteRecord;
use crate::repository::traits::InviteRepo;

pub struct InviteRepository {
    db: Arc<Surreal<Client>>,
}

impl InviteRepository {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl InviteRepo for InviteRepository {
    async fn create(&self, code: String, created_by: String, _expires_at: String) -> Result<(), surrealdb::Error> {
        self.db
            .query(
                "CREATE invite CONTENT {
                     code: $code,
                     created_by: $created_by,
                     expires_at: time::now() + 7d,
                     used: false,
                     created_at: time::now()
                 }",
            )
            .bind(("code", code))
            .bind(("created_by", created_by))
            .await?;
        Ok(())
    }

    async fn find_valid(&self, code: String) -> Result<Option<InviteRecord>, surrealdb::Error> {
        let mut res = self.db
            .query(
                "SELECT meta::id(id) as id, code, created_by, expires_at, used
                 FROM invite
                 WHERE code = $code AND used = false AND expires_at > time::now()
                 LIMIT 1",
            )
            .bind(("code", code))
            .await?;
        res.take(0)
    }

    async fn mark_used(&self, code: String) -> Result<(), surrealdb::Error> {
        self.db
            .query("UPDATE invite SET used = true WHERE code = $code")
            .bind(("code", code))
            .await?;
        Ok(())
    }
}
