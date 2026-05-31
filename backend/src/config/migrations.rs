use std::sync::Arc;

use serde::Deserialize;
use surrealdb::{Surreal, engine::remote::ws::Client};

type Db = Arc<Surreal<Client>>;

#[derive(Deserialize)]
struct CountResult {
    total: u32,
}

static MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_indexes",
        include_str!("../../migrations/0001_indexes.surql"),
    ),
    (
        "0002_schema",
        include_str!("../../migrations/0002_schema.surql"),
    ),
    (
        "0003_backfill_user_defaults",
        include_str!("../../migrations/0003_backfill_user_defaults.surql"),
    ),
    (
        "0004_fix_user_defaults",
        include_str!("../../migrations/0004_fix_user_defaults.surql"),
    ),
];

pub async fn run(db: &Db) -> Result<(), surrealdb::Error> {
    db.query("DEFINE TABLE IF NOT EXISTS _migration").await?;

    for (name, sql) in MIGRATIONS {
        let counts: Vec<CountResult> = db
            .query(
                "SELECT count() AS total FROM _migration
                 WHERE id = type::thing('_migration', $n)
                 GROUP ALL",
            )
            .bind(("n", *name))
            .await?
            .take(0)?;

        if counts.first().is_some_and(|c| c.total > 0) {
            tracing::debug!(migration = name, "Already applied, skipping");
            continue;
        }

        tracing::info!(migration = name, "Applying migration");

        // Run each statement individually so per-statement errors propagate.
        // Splitting on ';' and filtering blanks handles multi-statement files.
        for stmt in sql.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty() {
                continue;
            }
            db.query(stmt).await?.check().map_err(|e| {
                tracing::error!(migration = name, stmt, "Migration statement failed: {e}");
                e
            })?;
        }

        db.query(
            "INSERT INTO _migration { id: type::thing('_migration', $n), applied_at: time::now() }",
        )
        .bind(("n", *name))
        .await?;
        tracing::info!(migration = name, "Migration applied successfully");
    }

    Ok(())
}
