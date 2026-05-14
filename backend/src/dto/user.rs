use serde::Serialize;

use crate::model::user::UserSummary;

#[derive(Debug, Serialize)]
pub struct FriendResponse {
    pub keycloak_id: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub keycloak_id: String,
    pub username: String,
    pub games: Vec<String>,
    pub is_friend: bool,
    /// Only present when visibility allows it for the requesting user.
    pub steam_handle: Option<String>,
    pub discord_handle: Option<String>,
}

impl From<UserSummary> for FriendResponse {
    fn from(u: UserSummary) -> Self {
        FriendResponse {
            keycloak_id: u.keycloak_id,
            username: u.username,
        }
    }
}
