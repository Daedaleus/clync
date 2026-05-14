use std::sync::Arc;

use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::dto::me::{MeResponse, UpdateProfileRequest};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::repository::{
    group::GroupRepository,
    traits::{GroupRepo, UserRepo},
    user::UserRepository,
};

pub struct MeService {
    user_repo: Box<dyn UserRepo>,
    group_repo: Box<dyn GroupRepo>,
}

impl MeService {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self {
            user_repo: Box::new(UserRepository::new(Arc::clone(&db))),
            group_repo: Box::new(GroupRepository::new(db)),
        }
    }

    pub async fn get_me(&self, user: &AuthUser) -> Result<MeResponse, AppError> {
        let (profile, groups) = tokio::try_join!(
            self.user_repo.get_full_profile(user.keycloak_id.clone()),
            self.group_repo.find_by_member(user.keycloak_id.clone()),
        )?;

        let p = profile.unwrap_or_default();

        Ok(MeResponse {
            keycloak_id: user.keycloak_id.clone(),
            username: user.username.clone(),
            games: p.games,
            groups: groups.into_iter().map(Into::into).collect(),
            steam_handle: p.steam_handle,
            steam_visibility: p.steam_visibility,
            discord_handle: p.discord_handle,
            discord_visibility: p.discord_visibility,
        })
    }

    pub async fn update_profile(&self, keycloak_id: &str, req: UpdateProfileRequest) -> Result<(), AppError> {
        self.user_repo.update_social_profile(
            keycloak_id.to_owned(),
            req.steam_handle,
            req.steam_visibility,
            req.discord_handle,
            req.discord_visibility,
        ).await?;
        Ok(())
    }
}
