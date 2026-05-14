use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendRequest {
    pub id: String,
    pub from_id: String,
    pub from_username: String,
    pub to_id: String,
}
