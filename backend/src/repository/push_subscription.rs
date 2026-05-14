use std::sync::Arc;

use async_trait::async_trait;
use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::model::push_subscription::PushSubscription;
use crate::repository::traits::PushSubscriptionRepo;

pub struct PushSubscriptionRepository {
    db: Arc<Surreal<Client>>,
}

impl PushSubscriptionRepository {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PushSubscriptionRepo for PushSubscriptionRepository {
    async fn upsert(&self, sub: PushSubscription) -> Result<(), surrealdb::Error> {
        // Use user_id as record ID for easy lookup and deduplication per user
        self.db
            .upsert::<Option<PushSubscription>>(("push_subscription", sub.user_id.clone()))
            .content(sub)
            .await?;
        Ok(())
    }

    async fn delete(&self, user_id: String) -> Result<(), surrealdb::Error> {
        self.db
            .delete::<Option<PushSubscription>>(("push_subscription", user_id))
            .await?;
        Ok(())
    }

    async fn find_by_user_ids(
        &self,
        user_ids: Vec<String>,
    ) -> Result<Vec<PushSubscription>, surrealdb::Error> {
        if user_ids.is_empty() {
            return Ok(vec![]);
        }
        let mut res = self
            .db
            .query("SELECT * FROM push_subscription WHERE user_id IN $ids")
            .bind(("ids", user_ids))
            .await?;
        res.take(0)
    }
}
