use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

/// Full application configuration.
/// Loaded from (in order, later sources override earlier):
///   1. config/app.yaml        — base defaults (committed)
///   2. config/app.local.yaml  — local secrets/overrides (gitignored)
///   3. Environment variables  — prefix APP__, separator __ (for Docker)
///      e.g. APP__DATABASE__URL=ws://prod:8000
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: Server,
    pub database: Database,
    pub keycloak: Keycloak,
    pub vapid: Vapid,
    pub cors: Cors,
    pub rawg: Rawg,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub url: String,
    pub namespace: String,
    pub name: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Keycloak {
    pub jwks_uri: String,
    /// Base URL of the Keycloak server, e.g. http://localhost:8080
    pub admin_url: String,
    /// Realm name, e.g. Clync
    pub realm: String,
    pub admin_user: String,
    pub admin_password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Vapid {
    pub public_key: String,
    pub private_key: String,
    pub subject: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Cors {
    pub frontend_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rawg {
    pub api_key: String,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::with_name("config/app"))
            .add_source(File::with_name("config/app.local").required(false))
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize()
    }
}
