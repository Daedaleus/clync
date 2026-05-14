use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteRecord {
    pub id: String,
    pub code: String,
    pub created_by: String,
    pub expires_at: String,
    pub used: bool,
}
