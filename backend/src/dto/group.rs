use serde::{Deserialize, Serialize};

use crate::model::group::Group;

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub is_public: bool,
}

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub id: String,
    pub name: String,
    pub is_public: bool,
    pub member_count: usize,
}

#[derive(Debug, Serialize)]
pub struct GroupSummary {
    pub id: String,
    pub name: String,
    pub is_public: bool,
}

#[derive(Debug, Serialize)]
pub struct MemberResponse {
    pub keycloak_id: String,
    pub username: String,
    pub is_friend: bool,
}

#[derive(Debug, Serialize)]
pub struct PossibleGame {
    pub name: String,
    /// Number of members who have this game in their wishlist.
    pub count: usize,
    /// Total number of members in the group.
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct GroupDetailResponse {
    pub id: String,
    pub name: String,
    pub is_public: bool,
    pub creator_id: Option<String>,
    pub discord_invite: Option<String>,
    pub members: Vec<MemberResponse>,
    pub common_games: Vec<String>,
    /// All games any member has, sorted by count desc then name asc.
    pub possible_games: Vec<PossibleGame>,
}

#[derive(Debug, Deserialize)]
pub struct SetDiscordInviteRequest {
    /// `null` or omitted clears the link.
    pub url: Option<String>,
}

impl From<Group> for GroupResponse {
    fn from(g: Group) -> Self {
        GroupResponse {
            member_count: g.members.len(),
            id: g.id,
            name: g.name,
            is_public: g.is_public,
        }
    }
}

impl From<Group> for GroupSummary {
    fn from(g: Group) -> Self {
        GroupSummary {
            id: g.id,
            name: g.name,
            is_public: g.is_public,
        }
    }
}
