use serde::{Deserialize, Serialize};

use crate::dto::group::GroupSummary;
use crate::model::user::Visibility;

#[derive(Serialize)]
pub struct MeResponse {
    pub keycloak_id: String,
    pub username: String,
    pub games: Vec<String>,
    pub groups: Vec<GroupSummary>,
    pub steam_handle: Option<String>,
    pub steam_visibility: Visibility,
    pub discord_handle: Option<String>,
    pub discord_visibility: Visibility,
}

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub steam_handle: Option<String>,
    #[serde(default)]
    pub steam_visibility: Visibility,
    pub discord_handle: Option<String>,
    #[serde(default)]
    pub discord_visibility: Visibility,
}
