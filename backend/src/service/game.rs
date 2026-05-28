use std::sync::Arc;

use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::dto::game::{UpdateGameRequest, UpsertGameRequest};
use crate::error::{AppError, codes};
use crate::model::game::Game;
use crate::repository::{
    game::GameRepository,
    traits::{GameRepo, UserRepo},
    user::UserRepository,
};
use crate::service::rawg::{AutofillCandidate, RawgClient};

pub struct GameService {
    game_repo: Box<dyn GameRepo>,
    user_repo: Box<dyn UserRepo>,
}

impl GameService {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self {
            game_repo: Box::new(GameRepository::new(Arc::clone(&db))),
            user_repo: Box::new(UserRepository::new(db)),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_repos(game_repo: Box<dyn GameRepo>, user_repo: Box<dyn UserRepo>) -> Self {
        Self {
            game_repo,
            user_repo,
        }
    }

    pub async fn suggestions(&self, query: &str) -> Result<Vec<String>, AppError> {
        if query.trim().len() < 2 {
            return Ok(vec![]);
        }
        Ok(self.game_repo.search(query.to_owned()).await?)
    }

    pub async fn add(&self, keycloak_id: &str, name: &str) -> Result<(), AppError> {
        let name = name.trim().to_owned();
        if name.is_empty() {
            return Err(AppError::Validation(codes::game::NAME_REQUIRED.into()));
        }
        self.game_repo.ensure_exists(name.clone()).await?;
        self.user_repo
            .add_game(keycloak_id.to_owned(), name)
            .await?;
        Ok(())
    }

    pub async fn remove(&self, keycloak_id: &str, name: &str) -> Result<(), AppError> {
        self.user_repo
            .remove_game(keycloak_id.to_owned(), name.to_owned())
            .await?;
        // Auto-delete the library entry when the last user removes it.
        if !self.game_repo.is_in_any_wishlist(name.to_owned()).await? {
            let _ = self.game_repo.delete(name.to_owned()).await;
        }
        Ok(())
    }

    /// Deletes a game. Admin can always delete; others get an error if the game is in use.
    pub async fn delete_game(&self, name: &str, is_admin: bool) -> Result<(), AppError> {
        if !is_admin && self.game_repo.is_in_any_wishlist(name.to_owned()).await? {
            return Err(AppError::Validation(codes::game::IN_WISHLIST.into()));
        }
        self.game_repo.delete(name.to_owned()).await?;
        Ok(())
    }

    pub async fn list_all(&self) -> Result<Vec<Game>, AppError> {
        Ok(self.game_repo.list_all().await?)
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Option<Game>, AppError> {
        Ok(self.game_repo.find_by_name(name.to_owned()).await?)
    }

    pub async fn create(&self, req: UpsertGameRequest, creator_id: &str) -> Result<(), AppError> {
        let name = req.name.trim().to_owned();
        if name.is_empty() {
            return Err(AppError::Validation(codes::game::NAME_REQUIRED.into()));
        }
        self.game_repo
            .upsert_with_details(name.clone(), req.description, req.genre)
            .await?;
        self.user_repo.add_game(creator_id.to_owned(), name).await?;
        Ok(())
    }

    pub async fn store_thumbnail(
        &self,
        name: &str,
        data: Vec<u8>,
        content_type: String,
    ) -> Result<(), AppError> {
        self.game_repo
            .store_thumbnail(name.to_owned(), data, content_type)
            .await?;
        Ok(())
    }

    pub async fn get_thumbnail(&self, name: &str) -> Result<Option<(Vec<u8>, String)>, AppError> {
        Ok(self.game_repo.get_thumbnail(name.to_owned()).await?)
    }

    /// Step 1 — returns up to 3 RAWG search candidates for the user to pick from.
    pub async fn get_autofill_candidates(
        &self,
        name: &str,
        rawg_api_key: &str,
    ) -> Result<Vec<AutofillCandidate>, AppError> {
        if rawg_api_key.is_empty() {
            return Err(AppError::Validation(
                codes::game::RAWG_NOT_CONFIGURED.into(),
            ));
        }
        RawgClient::new(rawg_api_key.to_owned())
            .search_candidates(name, 3)
            .await
    }

    /// Step 2 — fetches full detail for the chosen RAWG ID and persists it.
    /// Also adds the game to the requester's wishlist.
    pub async fn confirm_autofill(
        &self,
        name: &str,
        rawg_id: u64,
        rawg_api_key: &str,
        genre: Option<String>,
        requester_id: &str,
    ) -> Result<Game, AppError> {
        if rawg_api_key.is_empty() {
            return Err(AppError::Validation(
                codes::game::RAWG_NOT_CONFIGURED.into(),
            ));
        }

        let data = RawgClient::new(rawg_api_key.to_owned())
            .fetch_detail(rawg_id)
            .await?;

        self.game_repo
            .upsert_with_details(name.to_owned(), data.description, genre)
            .await?;

        if let Some((bytes, ct)) = data.thumbnail_bytes {
            self.game_repo
                .store_thumbnail(name.to_owned(), bytes, ct)
                .await?;
        }

        self.user_repo
            .add_game(requester_id.to_owned(), name.to_owned())
            .await?;

        self.game_repo
            .find_by_name(name.to_owned())
            .await?
            .ok_or_else(|| AppError::Internal("game not found after autofill".into()))
    }

    pub async fn update(&self, name: &str, req: UpdateGameRequest) -> Result<(), AppError> {
        let existing = self
            .game_repo
            .find_by_name(name.to_owned())
            .await?
            .ok_or_else(|| AppError::Validation(codes::game::NOT_FOUND.into()))?;
        self.game_repo
            .upsert_with_details(existing.name, req.description, req.genre)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::user::{User, UserSummary};
    use async_trait::async_trait;

    struct PanicGameRepo;
    struct PanicUserRepo;

    #[async_trait]
    impl GameRepo for PanicGameRepo {
        async fn search(&self, _: String) -> Result<Vec<String>, surrealdb::Error> {
            panic!()
        }
        async fn ensure_exists(&self, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn list_all(&self) -> Result<Vec<crate::model::game::Game>, surrealdb::Error> {
            panic!()
        }
        async fn find_by_name(
            &self,
            _: String,
        ) -> Result<Option<crate::model::game::Game>, surrealdb::Error> {
            panic!()
        }
        async fn upsert_with_details(
            &self,
            _: String,
            _: Option<String>,
            _: Option<String>,
        ) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn store_thumbnail(
            &self,
            _: String,
            _: Vec<u8>,
            _: String,
        ) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn get_thumbnail(
            &self,
            _: String,
        ) -> Result<Option<(Vec<u8>, String)>, surrealdb::Error> {
            panic!()
        }
        async fn is_in_any_wishlist(&self, _: String) -> Result<bool, surrealdb::Error> {
            panic!()
        }
        async fn delete(&self, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
    }

    #[async_trait]
    impl UserRepo for PanicUserRepo {
        async fn upsert(&self, _: User) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn search(&self, _: String, _: String) -> Result<Vec<UserSummary>, surrealdb::Error> {
            panic!()
        }
        async fn find_by_ids(&self, _: Vec<String>) -> Result<Vec<UserSummary>, surrealdb::Error> {
            panic!()
        }
        async fn add_game(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn remove_game(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn get_friend_ids(&self, _: String) -> Result<Vec<String>, surrealdb::Error> {
            panic!()
        }
        async fn add_friend(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn remove_friend(&self, _: String, _: String) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn get_full_profile(
            &self,
            _: String,
        ) -> Result<Option<crate::model::user::UserFullProfile>, surrealdb::Error> {
            panic!()
        }
        async fn update_social_profile(
            &self,
            _: String,
            _: Option<String>,
            _: crate::model::user::Visibility,
            _: Option<String>,
            _: crate::model::user::Visibility,
        ) -> Result<(), surrealdb::Error> {
            panic!()
        }
        async fn find_who_added(
            &self,
            _: String,
            _: Vec<String>,
        ) -> Result<Vec<crate::model::user::UserSummary>, surrealdb::Error> {
            panic!()
        }
    }

    fn svc() -> GameService {
        GameService::with_repos(Box::new(PanicGameRepo), Box::new(PanicUserRepo))
    }

    #[tokio::test]
    async fn suggestions_with_one_char_returns_empty() {
        let result = svc().suggestions("a").await;
        assert!(matches!(result, Ok(v) if v.is_empty()));
    }

    #[tokio::test]
    async fn suggestions_with_empty_returns_empty() {
        let result = svc().suggestions("").await;
        assert!(matches!(result, Ok(v) if v.is_empty()));
    }

    #[tokio::test]
    async fn add_empty_name_returns_error() {
        assert!(svc().add("user1", "").await.is_err());
    }

    #[tokio::test]
    async fn add_whitespace_only_name_returns_error() {
        assert!(svc().add("user1", "   ").await.is_err());
    }
}
