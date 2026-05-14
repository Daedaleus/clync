use std::sync::Arc;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::model::game::Game;
use crate::repository::traits::GameRepo;

pub struct GameRepository {
    db: Arc<Surreal<Client>>,
}

impl GameRepository {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }
}

/// Internal DB row — includes thumbnail fields that are not exposed in the API.
#[derive(Deserialize)]
struct GameRow {
    name: String,
    description: Option<String>,
    genre: Option<String>,
    /// Presence of this field indicates a thumbnail exists.
    thumbnail_b64: Option<String>,
}

impl GameRow {
    fn into_game(self) -> Game {
        let has_thumb = self.thumbnail_b64.is_some();
        Game {
            thumbnail_url: has_thumb.then(|| format!("/api/v1/library/{}/thumbnail", self.name)),
            name: self.name,
            description: self.description,
            genre: self.genre,
        }
    }
}

#[derive(Deserialize)]
struct ThumbnailRow {
    thumbnail_b64: Option<String>,
    thumbnail_content_type: Option<String>,
}

#[async_trait]
impl GameRepo for GameRepository {
    async fn search(&self, query: String) -> Result<Vec<String>, surrealdb::Error> {
        let mut res = self.db
            .query(
                "SELECT VALUE name FROM game
                 WHERE string::contains(string::lowercase(name), string::lowercase($q))
                 ORDER BY name LIMIT 10",
            )
            .bind(("q", query))
            .await?;
        res.take(0)
    }

    async fn ensure_exists(&self, name: String) -> Result<(), surrealdb::Error> {
        self.db
            .query("UPSERT type::thing('game', $name) MERGE { name: $name }")
            .bind(("name", name))
            .await?;
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<Game>, surrealdb::Error> {
        let mut res = self.db
            .query("SELECT name, description, genre, thumbnail_b64 FROM game ORDER BY name LIMIT 500")
            .await?;
        let rows: Vec<GameRow> = res.take(0)?;
        Ok(rows.into_iter().map(|r| r.into_game()).collect())
    }

    async fn find_by_name(&self, name: String) -> Result<Option<Game>, surrealdb::Error> {
        let mut res = self.db
            .query(
                "SELECT name, description, genre, thumbnail_b64
                 FROM type::thing('game', $name) LIMIT 1",
            )
            .bind(("name", name))
            .await?;
        let row: Option<GameRow> = res.take(0)?;
        Ok(row.map(|r| r.into_game()))
    }

    async fn upsert_with_details(
        &self,
        name: String,
        description: Option<String>,
        genre: Option<String>,
    ) -> Result<(), surrealdb::Error> {
        self.db
            .query(
                "UPSERT type::thing('game', $name) MERGE {
                     name: $name,
                     description: $description,
                     genre: $genre
                 }",
            )
            .bind(("name", name))
            .bind(("description", description))
            .bind(("genre", genre))
            .await?;
        Ok(())
    }

    async fn store_thumbnail(&self, name: String, data: Vec<u8>, content_type: String) -> Result<(), surrealdb::Error> {
        let b64 = STANDARD.encode(&data);
        self.db
            .query(
                "UPDATE type::thing('game', $name) MERGE {
                     thumbnail_b64: $b64,
                     thumbnail_content_type: $ct
                 }",
            )
            .bind(("name", name))
            .bind(("b64", b64))
            .bind(("ct", content_type))
            .await?;
        Ok(())
    }

    async fn is_in_any_wishlist(&self, name: String) -> Result<bool, surrealdb::Error> {
        let mut res = self.db
            .query("SELECT id FROM user WHERE $name IN (games ?? []) LIMIT 1")
            .bind(("name", name))
            .await?;
        let rows: Vec<serde_json::Value> = res.take(0)?;
        Ok(!rows.is_empty())
    }

    async fn delete(&self, name: String) -> Result<(), surrealdb::Error> {
        self.db
            .query("DELETE type::thing('game', $name)")
            .bind(("name", name))
            .await?;
        Ok(())
    }

    async fn get_thumbnail(&self, name: String) -> Result<Option<(Vec<u8>, String)>, surrealdb::Error> {
        let mut res = self.db
            .query(
                "SELECT thumbnail_b64, thumbnail_content_type
                 FROM type::thing('game', $name) LIMIT 1",
            )
            .bind(("name", name))
            .await?;
        let row: Option<ThumbnailRow> = res.take(0)?;
        match row {
            Some(ThumbnailRow { thumbnail_b64: Some(b64), thumbnail_content_type: Some(ct) }) => {
                let bytes = STANDARD.decode(&b64).unwrap_or_default();
                Ok(Some((bytes, ct)))
            }
            _ => Ok(None),
        }
    }
}
