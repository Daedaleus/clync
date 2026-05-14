use std::sync::Arc;

use surrealdb::{
    Surreal,
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
};

use crate::config::settings::Database;

pub async fn connect(cfg: &Database) -> Result<Arc<Surreal<Client>>, surrealdb::Error> {
    let db = Surreal::new::<Ws>(cfg.url.clone()).await?;
    db.signin(Root { username: &cfg.username, password: &cfg.password }).await?;
    db.use_ns(cfg.namespace.clone()).use_db(cfg.name.clone()).await?;
    tracing::info!("Connected to SurrealDB at {}", cfg.url);
    Ok(Arc::new(db))
}
