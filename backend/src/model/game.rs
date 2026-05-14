use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub name: String,
    pub description: Option<String>,
    pub genre: Option<String>,
    pub thumbnail_url: Option<String>,
}
