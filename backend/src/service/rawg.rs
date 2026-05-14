use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutofillCandidate {
    pub rawg_id: u64,
    pub rawg_name: String,
    pub thumbnail_url: Option<String>,
    pub genre: Option<String>,
}

pub struct RawgGameData {
    pub description: Option<String>,
    pub thumbnail_bytes: Option<(Vec<u8>, String)>,
}

// ── Internal deserialization structs ─────────────────────────────────────────

#[derive(Deserialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
}

#[derive(Deserialize)]
struct SearchResult {
    id: u64,
    name: String,
    background_image: Option<String>,
    genres: Vec<Genre>,
}

#[derive(Deserialize)]
struct Genre {
    name: String,
}

#[derive(Deserialize)]
struct DetailResponse {
    description_raw: Option<String>,
    background_image: Option<String>,
}

// ── Client ────────────────────────────────────────────────────────────────────

pub struct RawgClient {
    api_key: String,
    client: reqwest::Client,
}

impl RawgClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key, client: reqwest::Client::new() }
    }

    /// Returns up to `count` search candidates for display in a picker.
    pub async fn search_candidates(&self, name: &str, count: usize) -> Result<Vec<AutofillCandidate>, AppError> {
        let search: SearchResponse = self.client
            .get("https://api.rawg.io/api/games")
            .query(&[
                ("key", self.api_key.as_str()),
                ("search", name),
                ("page_size", &count.to_string()),
            ])
            .send().await
            .map_err(|e| AppError::Internal(format!("RAWG search failed: {e}")))?
            .json().await
            .map_err(|e| AppError::Internal(format!("RAWG parse error: {e}")))?;

        Ok(search.results.into_iter().map(|r| AutofillCandidate {
            rawg_id: r.id,
            rawg_name: r.name,
            thumbnail_url: r.background_image,
            genre: r.genres.into_iter().next().map(|g| g.name),
        }).collect())
    }

    /// Fetches full game detail for a specific RAWG ID.
    pub async fn fetch_detail(&self, rawg_id: u64) -> Result<RawgGameData, AppError> {
        let detail: DetailResponse = self.client
            .get(format!("https://api.rawg.io/api/games/{rawg_id}"))
            .query(&[("key", self.api_key.as_str())])
            .send().await
            .map_err(|e| AppError::Internal(format!("RAWG detail failed: {e}")))?
            .json().await
            .map_err(|e| AppError::Internal(format!("RAWG detail parse error: {e}")))?;

        let description = detail.description_raw.and_then(|d| {
            d.lines()
                .map(str::trim)
                .find(|l| !l.is_empty())
                .map(str::to_owned)
        });

        let thumbnail_bytes = if let Some(url) = detail.background_image {
            download_image(&self.client, &url).await.ok().flatten()
        } else {
            None
        };

        Ok(RawgGameData { description, thumbnail_bytes })
    }
}

async fn download_image(client: &reqwest::Client, url: &str) -> Result<Option<(Vec<u8>, String)>, reqwest::Error> {
    let resp = client.get(url).send().await?;
    let ct = resp.headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_owned();
    let bytes = resp.bytes().await?.to_vec();
    Ok(Some((bytes, ct)))
}
