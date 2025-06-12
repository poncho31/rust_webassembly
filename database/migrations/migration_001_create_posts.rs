use anyhow::Result;
use crate::repositories::_database_query::DatabaseQuery;

/// Migration pour créer la table "posts" et ajouter des index
pub async fn migrate(repo: &DatabaseQuery) -> Result<()> {
    // Création de la table posts
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS logs (
            id          SERIAL PRIMARY KEY,
            type        TEXT NOT NULL,
            message     TEXT NOT NULL,
            level       TEXT NOT NULL,
            created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#;
    repo.run_query(create_table_query).await?;

    // Création des index sur title et author_id
    repo.create_indexes("posts", vec!["type", "level", "created_at"]).await?;

    Ok(())
}