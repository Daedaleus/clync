use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionInvitationRecord {
    pub id: String,
    pub session_id: String,
    pub game: String,
    pub scheduled_at: String,
    pub inviter_id: String,
    pub inviter_username: String,
    pub invitee_id: String,
}
