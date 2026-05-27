use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsvpRecord {
    pub user_id: String,
    pub username: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub game: String,
    pub scheduled_at: String,
    pub scope: String,
    #[serde(default)]
    pub group_ids: Vec<String>,
    #[serde(default)]
    pub participants: Vec<String>,
    #[serde(default)]
    pub group_names: Vec<String>,
    #[serde(default)]
    pub game_has_thumbnail: bool,
    pub notes: Option<String>,
    #[serde(default)]
    pub rsvps: Vec<RsvpRecord>,
}

/// Minimal projection used by the start-notification background task.
#[derive(Debug, serde::Deserialize)]
pub struct SessionStartReminder {
    pub id: String,
    pub user_id: String,
    #[serde(default)]
    pub participants: Vec<String>,
    pub game: String,
}

/// Session with participant names resolved — used for the detail endpoint.
#[derive(Debug, Deserialize)]
pub struct SessionDetail {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub game: String,
    pub scheduled_at: String,
    pub scope: String,
    #[serde(default)]
    pub group_ids: Vec<String>,
    #[serde(default)]
    pub group_names: Vec<String>,
    #[serde(default)]
    pub game_has_thumbnail: bool,
    #[serde(default)]
    pub participants: Vec<ParticipantRecord>,
    pub notes: Option<String>,
    #[serde(default)]
    pub rsvps: Vec<RsvpRecord>,
}

#[derive(Debug, Deserialize)]
pub struct ParticipantRecord {
    pub keycloak_id: String,
    pub username: String,
}
