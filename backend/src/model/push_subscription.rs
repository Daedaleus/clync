use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    pub user_id: String,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}
