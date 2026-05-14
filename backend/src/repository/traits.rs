use async_trait::async_trait;

use crate::model::{
    game::Game,
    group::{Group, MemberWithGames},
    invitation::InvitationRecord,
    invite::InviteRecord,
    push_subscription::PushSubscription,
    session::{Session, SessionDetail},
    user::{User, UserFullProfile, UserSummary, Visibility},
};

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn upsert(&self, user: User) -> Result<(), surrealdb::Error>;
    async fn search(
        &self,
        query: String,
        exclude_id: String,
    ) -> Result<Vec<UserSummary>, surrealdb::Error>;
    async fn find_by_ids(&self, ids: Vec<String>) -> Result<Vec<UserSummary>, surrealdb::Error>;
    async fn add_game(
        &self,
        keycloak_id: String,
        game_name: String,
    ) -> Result<(), surrealdb::Error>;
    async fn remove_game(
        &self,
        keycloak_id: String,
        game_name: String,
    ) -> Result<(), surrealdb::Error>;
    async fn get_friend_ids(&self, keycloak_id: String) -> Result<Vec<String>, surrealdb::Error>;
    async fn add_friend(&self, user_id: String, friend_id: String) -> Result<(), surrealdb::Error>;
    async fn remove_friend(
        &self,
        user_id: String,
        friend_id: String,
    ) -> Result<(), surrealdb::Error>;
    async fn get_full_profile(
        &self,
        keycloak_id: String,
    ) -> Result<Option<UserFullProfile>, surrealdb::Error>;
    async fn update_social_profile(
        &self,
        keycloak_id: String,
        steam_handle: Option<String>,
        steam_visibility: Visibility,
        discord_handle: Option<String>,
        discord_visibility: Visibility,
    ) -> Result<(), surrealdb::Error>;
    /// Returns members of `candidate_ids` who have `user_id` in their own friends list (i.e. they added the user back).
    async fn find_who_added(
        &self,
        user_id: String,
        candidate_ids: Vec<String>,
    ) -> Result<Vec<UserSummary>, surrealdb::Error>;
}

#[async_trait]
pub trait GroupRepo: Send + Sync {
    async fn create(
        &self,
        name: String,
        is_public: bool,
        creator_id: String,
    ) -> Result<Option<Group>, surrealdb::Error>;
    async fn find_by_id(&self, group_id: String) -> Result<Option<Group>, surrealdb::Error>;
    async fn find_member_games(
        &self,
        member_ids: Vec<String>,
    ) -> Result<Vec<MemberWithGames>, surrealdb::Error>;
    async fn list_public(&self) -> Result<Vec<Group>, surrealdb::Error>;
    async fn search_public(&self, query: String) -> Result<Vec<Group>, surrealdb::Error>;
    async fn find_by_member(&self, keycloak_id: String) -> Result<Vec<Group>, surrealdb::Error>;
    async fn join(&self, group_id: String, keycloak_id: String) -> Result<(), surrealdb::Error>;
    async fn share_group(&self, user_a: String, user_b: String) -> Result<bool, surrealdb::Error>;
    async fn delete(&self, group_id: String) -> Result<(), surrealdb::Error>;
    async fn find_all(&self) -> Result<Vec<Group>, surrealdb::Error>;
    async fn set_discord_invite(&self, group_id: String, url: Option<String>) -> Result<(), surrealdb::Error>;
}

#[async_trait]
pub trait GameRepo: Send + Sync {
    async fn search(&self, query: String) -> Result<Vec<String>, surrealdb::Error>;
    /// Creates a minimal game record if it doesn't exist yet; never overwrites existing metadata.
    async fn ensure_exists(&self, name: String) -> Result<(), surrealdb::Error>;
    async fn list_all(&self) -> Result<Vec<Game>, surrealdb::Error>;
    async fn find_by_name(&self, name: String) -> Result<Option<Game>, surrealdb::Error>;
    async fn upsert_with_details(
        &self,
        name: String,
        description: Option<String>,
        genre: Option<String>,
    ) -> Result<(), surrealdb::Error>;
    /// Stores thumbnail bytes (base64) and content-type in the game record.
    async fn store_thumbnail(
        &self,
        name: String,
        data: Vec<u8>,
        content_type: String,
    ) -> Result<(), surrealdb::Error>;
    /// Returns (bytes, content_type) if a thumbnail is stored, else None.
    async fn get_thumbnail(
        &self,
        name: String,
    ) -> Result<Option<(Vec<u8>, String)>, surrealdb::Error>;
    /// Returns true if at least one user has this game in their wishlist.
    async fn is_in_any_wishlist(&self, name: String) -> Result<bool, surrealdb::Error>;
    async fn delete(&self, name: String) -> Result<(), surrealdb::Error>;
}

#[async_trait]
pub trait SessionRepo: Send + Sync {
    async fn create(
        &self,
        user_id: String,
        username: String,
        game: String,
        scheduled_at: String,
        scope: String,
        group_ids: Vec<String>,
    ) -> Result<Option<Session>, surrealdb::Error>;
    async fn find_feed(&self, my_group_ids: Vec<String>) -> Result<Vec<Session>, surrealdb::Error>;
    async fn find_mine(&self, user_id: String) -> Result<Vec<Session>, surrealdb::Error>;
    async fn find_for_group(&self, group_id: String) -> Result<Vec<Session>, surrealdb::Error>;
    async fn find_by_id(
        &self,
        session_id: String,
    ) -> Result<Option<SessionDetail>, surrealdb::Error>;
    async fn join(
        &self,
        session_id: String,
        user_id: String,
    ) -> Result<Option<Session>, surrealdb::Error>;
    async fn leave(&self, session_id: String, user_id: String) -> Result<(), surrealdb::Error>;
    async fn delete(
        &self,
        session_id: String,
        user_id: String,
    ) -> Result<Vec<String>, surrealdb::Error>;
    /// Admin-only delete — skips owner check, always returns group_ids.
    async fn delete_as_admin(&self, session_id: String) -> Result<Vec<String>, surrealdb::Error>;
}

#[async_trait]
pub trait InviteRepo: Send + Sync {
    async fn create(
        &self,
        code: String,
        created_by: String,
        expires_at: String,
    ) -> Result<(), surrealdb::Error>;
    async fn find_valid(&self, code: String) -> Result<Option<InviteRecord>, surrealdb::Error>;
    async fn mark_used(&self, code: String) -> Result<(), surrealdb::Error>;
}

#[async_trait]
pub trait InvitationRepo: Send + Sync {
    async fn create(
        &self,
        group_id: String,
        group_name: String,
        inviter_id: String,
        inviter_username: String,
        invitee_id: String,
    ) -> Result<(), surrealdb::Error>;
    async fn find_pending_for_user(
        &self,
        user_id: String,
    ) -> Result<Vec<InvitationRecord>, surrealdb::Error>;
    async fn find_by_id_for_invitee(
        &self,
        inv_id: String,
        user_id: String,
    ) -> Result<Option<InvitationRecord>, surrealdb::Error>;
    async fn exists_for_group_and_invitee(
        &self,
        group_id: String,
        invitee_id: String,
    ) -> Result<bool, surrealdb::Error>;
    async fn delete(&self, inv_id: String, user_id: String) -> Result<(), surrealdb::Error>;
}

#[async_trait]
pub trait PushSubscriptionRepo: Send + Sync {
    async fn upsert(&self, sub: PushSubscription) -> Result<(), surrealdb::Error>;
    async fn delete(&self, user_id: String) -> Result<(), surrealdb::Error>;
    async fn find_by_user_ids(
        &self,
        user_ids: Vec<String>,
    ) -> Result<Vec<PushSubscription>, surrealdb::Error>;
}
