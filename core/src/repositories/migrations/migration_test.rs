use anyhow::Result;
use crate::repositories::_database_query::DatabaseQuery;
use crate::repositories::_init_repository::InitRepository;


const TABLE   : &str   = "tests";
const INDEXES: &[&str] = &["level", "created_at"];

/// Migration pour créer la table "posts" et ajouter des index
pub async fn migrate(repo: &DatabaseQuery) -> Result<()> {
    println!("Running migration to create logs table...");

    let table_fields = r#"
        id          UUID PRIMARY KEY,
        level       INT NOT NULL,
        message     TEXT NOT NULL,
        context     TEXT,
        created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
    "#;

    // Création de la table posts
    repo.create_tables(
        TABLE,
        table_fields
    ).await?;


    // Création des index
    repo.create_indexes(TABLE, INDEXES.to_vec()).await?;

    println!("Migration to create logs table completed successfully.");   // Créer une instance de InitRepository et appeler la méthode init_repository
   let init_repo = InitRepository::new(repo.get_pool().clone());
   init_repo.init_repository(TABLE, table_fields).await?;
   
   Ok(())
}

pub async fn rollback(repo: &DatabaseQuery) -> Result<()> {
    println!("Rolling back migration by dropping logs table...");


    // Suppression de la table posts
    repo.drop_indexes(TABLE, INDEXES.to_vec()).await?;
    repo.drop_table(TABLE).await?;

    println!("Rollback completed successfully.");
    Ok(())
}