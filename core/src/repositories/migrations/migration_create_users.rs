use anyhow::{Result, bail};
use crate::repositories::_database::DatabaseQuery;
use crate::repositories::_init_repository::InitRepository;
use crate::repositories::migration_repository::{MigrationRepository, Migration};


const TABLE   : &str   = "users";
const INDEXES: &[&str] = &["login", "email", "created_at"];
const DESCRIPTION: Option<&str> = Some("Migration to create the users table");
const MIGRATION_NAME : &str = "create_users";


pub async fn run(repo: &DatabaseQuery) -> Result<()> {
    println!("Running migration '{}'...", MIGRATION_NAME);
    let repo_migration = MigrationRepository::new(repo.clone());

      // Vérifie si la migration existe déjà en base de données
    let migration_result = repo_migration.find_by_name(MIGRATION_NAME).await?;

    if migration_result.is_none() {
        // Exécuter la migration
        migrate(repo).await?;

        // Enregistrer la migration dans la base de données
        let model = &Migration::new(&MIGRATION_NAME, DESCRIPTION);
        let create_migration = repo_migration.create(model).await?;
        println!("create migration '{:?}'", create_migration);
    }

    Ok(())
}


/// users pour créer la table "posts" et ajouter des index
pub async fn migrate(repo: &DatabaseQuery) -> Result<()> {
    let table_fields = r#"
                    id          UUID PRIMARY KEY,
                    login       TEXT,
                    birthday    TEXT,
                    firstname   TEXT,
                    lastname    TEXT,
                    sexe        TEXT,
                    age         INTEGER,
                    info        TEXT,
                    email       TEXT,
                    files_info  TEXT,
                    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
    "#;

    // Création de la table posts
    repo.create_tables(TABLE,table_fields).await?;


    // Création des index
    repo.create_indexes(TABLE, INDEXES.to_vec()).await?;

    Ok(())
}

pub async fn rollback(repo: &DatabaseQuery) -> Result<()> {
    // Suppression de la table posts
    repo.drop_indexes(TABLE, INDEXES.to_vec()).await?;
    repo.drop_table(TABLE).await?;
    Ok(())
}