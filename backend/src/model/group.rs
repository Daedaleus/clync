use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub is_public: bool,
    pub members: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct MemberWithGames {
    pub keycloak_id: String,
    pub username: String,
    #[serde(default)]
    pub games: Vec<String>,
}
