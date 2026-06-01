use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    #[default]
    Public,
    Group,
    Friends,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub keycloak_id: String,
    pub username: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    pub keycloak_id: String,
    pub username: String,
}

/// Full profile including social fields — used for own-profile reads.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UserFullProfile {
    pub keycloak_id: String,
    pub username: String,
    #[serde(default)]
    pub games: Vec<String>,
    pub steam_handle: Option<String>,
    #[serde(default)]
    pub steam_visibility: Visibility,
    pub discord_handle: Option<String>,
    #[serde(default)]
    pub discord_visibility: Visibility,
}
