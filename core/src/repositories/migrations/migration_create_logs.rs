use anyhow::Result;
use crate::repositories::_database_query::DatabaseQuery;


const TABLE: &str = "logs";

/// Migration pour créer la table "posts" et ajouter des index
pub async fn migrate(repo: &DatabaseQuery) -> Result<()> {
    println!("Running migration to create logs table...");


    // Création de la table posts
    repo.create_tables(
        TABLE,
        r#"
            id          UUID PRIMARY KEY,
            type        TEXT NOT NULL,
            level       TEXT NOT NULL,
            message     TEXT NOT NULL,
            context     TEXT,
            created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
        "#
    ).await?;


    // Création des index
    repo.create_indexes(TABLE, vec!["type", "level", "created_at"]).await?;

    println!("Migration to create logs table completed successfully.");
    Ok(())
}

pub async fn rollback(repo: &DatabaseQuery) -> Result<()> {
    println!("Rolling back migration by dropping logs table...");


    // Suppression de la table posts
    repo.run_query(
        &format!("DROP TABLE IF EXISTS {}", TABLE)
    ).await?;


    println!("Rollback completed successfully.");
    Ok(())
}