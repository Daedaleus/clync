use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddGameRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RemoveGameQuery {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertGameRequest {
    pub name: String,
    pub description: Option<String>,
    pub genre: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGameRequest {
    pub description: Option<String>,
    pub genre: Option<String>,
}
