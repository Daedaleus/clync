use serde::{Deserialize, Serialize};

use crate::model::session::{Session, SessionDetail};

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub game: String,
    /// ISO 8601 UTC string from frontend (e.g. "2026-05-20T18:00:00.000Z")
    pub scheduled_at: String,
    /// "global" or "groups"
    pub scope: String,
    #[serde(default)]
    pub group_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub game: String,
    pub scheduled_at: String,
    pub scope: String,
    pub group_ids: Vec<String>,
    pub group_names: Vec<String>,
    /// 1 (creator) + joined participants
    pub participant_count: usize,
    pub is_mine: bool,
    pub is_participant: bool,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParticipantInfo {
    pub keycloak_id: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct SessionDetailResponse {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub game: String,
    pub scheduled_at: String,
    pub scope: String,
    pub group_ids: Vec<String>,
    pub group_names: Vec<String>,
    pub participants: Vec<ParticipantInfo>,
    pub participant_count: usize,
    pub is_mine: bool,
    pub is_participant: bool,
    pub thumbnail_url: Option<String>,
}

impl SessionDetail {
    pub fn into_detail_response(self, requester_id: &str) -> SessionDetailResponse {
        let is_participant = self.participants.iter().any(|p| p.keycloak_id == requester_id);
        let participant_count = 1 + self.participants.len();
        let thumbnail_url = if self.game_has_thumbnail {
            Some(format!("/api/v1/library/{}/thumbnail", self.game))
        } else {
            None
        };
        SessionDetailResponse {
            is_mine: self.user_id == requester_id,
            is_participant,
            participant_count,
            group_names: self.group_names,
            thumbnail_url,
            id: self.id,
            user_id: self.user_id,
            username: self.username,
            game: self.game,
            scheduled_at: self.scheduled_at,
            scope: self.scope,
            group_ids: self.group_ids,
            participants: self.participants.into_iter()
                .map(|p| ParticipantInfo { keycloak_id: p.keycloak_id, username: p.username })
                .collect(),
        }
    }
}

impl Session {
    pub fn into_response(self, requester_id: &str) -> SessionResponse {
        let thumbnail_url = if self.game_has_thumbnail {
            Some(format!("/api/v1/library/{}/thumbnail", self.game))
        } else {
            None
        };
        SessionResponse {
            is_mine: self.user_id == requester_id,
            is_participant: self.participants.contains(&requester_id.to_string()),
            participant_count: 1 + self.participants.len(),
            group_names: self.group_names,
            thumbnail_url,
            id: self.id,
            user_id: self.user_id,
            username: self.username,
            game: self.game,
            scheduled_at: self.scheduled_at,
            scope: self.scope,
            group_ids: self.group_ids,
        }
    }
}
