use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct InvitationRecord {
    pub id: String,
    pub group_id: String,
    pub group_name: String,
    pub inviter_id: String,
    pub inviter_username: String,
    pub invitee_id: String,
}
