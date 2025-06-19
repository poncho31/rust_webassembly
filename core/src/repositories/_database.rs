use sqlx::{query, PgPool, Row};
use anyhow::{Error, Result};
use uuid::Uuid;
use time::OffsetDateTime;
use std::collections::HashMap;
use sqlx::{pool, postgres::PgPoolOptions, Pool, Postgres};
use std::env;
use crate::repositories::migration_repository::{MigrationRepository, Migration};
use crate::repositories::{_init_repository::InitRepository};
use crate::repositories::migrations;

/// Représente une requête de base de données
pub struct DatabaseQuery {
    pool: PgPool,
}
/// Implémentation de la structure DatabaseQuery
/// Cette structure encapsule un pool de connexions à la base de données PostgreSQL
impl DatabaseQuery {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    // Méthode pour récupérer une référence au pool de connexion
    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }   
    
    /// Lance une requête en SQL brut
    pub async fn run_query(&self, query: &str) -> Result<()> {
        sqlx::query(query)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::msg(format!("\x1b[31mFailed query {}\x1b[0m", e)))?;

        println!("\x1b[32mQuery executed successfully: {}\x1b[0m", query);
        Ok(())
    }
    
    /// Exécute une requête de création de table
    pub async fn create_tables(&self, table_name: &str, columns: &str) -> Result<()> {
        let create_table_query = format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            table_name, columns
        );
        
        self.run_query(&create_table_query).await?;

        println!("Table {} created successfully", table_name);
        Ok(())
    }    
    
    /// Exécute une requête de suppression de table
    pub async fn drop_table(&self, table: &str) -> Result<()> {
        let drop_table_query = format!(
            "DROP TABLE IF EXISTS {}",
            table
        );
        self.run_query(&drop_table_query).await?;

        println!("Table dropped successfully");
        Ok(())
    }    
    
    /// Exécute une requête de création d'index
    pub async fn create_indexes(&self, table_name: &str, indexes: Vec<&str>) -> Result<()> {
        for idx in indexes {
            let index_query = format!(
                "CREATE INDEX IF NOT EXISTS idx_{}_{} ON {} ({})",
                table_name, idx, table_name, idx
            );
            self.run_query(&index_query).await?;
        }

        println!("Indexes created successfully for table: {}", table_name);
        Ok(())
    }    
    
    /// Exécute une requête de suppression d'index
    pub async fn drop_indexes(&self, table_name: &str, indexes: Vec<&str>) -> Result<()> {
        for idx in indexes {
            let drop_index_query = format!(
                "DROP INDEX IF EXISTS idx_{}_{}",
                table_name, idx
            );
            // Utilisation de run_query et propagation correcte des erreurs avec ?
            self.run_query(&drop_index_query).await?;
        }

        println!("Indexes dropped successfully for table: {}", table_name);
        Ok(())
    }
}


/// Exécute les migrations nécessaires pour créer les tables de base de données
pub async fn _init_migration_tables(pool: &Pool<Postgres>) {
    // Exécution des migrations via le module migrations
    println!("Running migrations...");

    // Exécution des migrations via le module migrations
    let repo: DatabaseQuery = DatabaseQuery::new(pool.clone());
    
    // Exécution des migrations avec gestion des erreurs
    if let Err(err) = migrations::migration_create_migration::run(&repo).await {
        eprintln!("Migration failed: {}", err);
    }
    
    // Continuer avec les autres migrations seulement si la première réussit
    if let Err(err) = migrations::migration_create_users::run(&repo).await {
        eprintln!("Migration failed: {}", err);
    }
    
    if let Err(err) = migrations::migration_create_logs::run(&repo).await {
        eprintln!("Migration failed: {}", err);
    }

    if let Err(err) = migrations::migration_test::run(&repo).await {
        eprintln!("Migration failed: {}", err);
    }

    println!("All migrations completed successfully.");
}

/// Initialise la base de données en créant un pool de connexions
pub async fn init_db() -> Result<Pool<Postgres>, Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    for i in 1..=3 {
        match PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await {
                Ok(pool) => {
                    // Créer une instance de DatabaseQuery pour utiliser run_query
                    let db_query = DatabaseQuery::new(pool.clone());
                    if let Ok(_) = db_query.run_query("SELECT 1").await {
                        // Initialize and create tables if needed
                        _init_migration_tables(&pool).await;

                        return Ok(pool);
                    }
                },
                Err(e) => {
                    println!("Connection attempt {} failed: {}", i, e);
                    if i < 3 {
                        println!("Retrying in 2 seconds...");
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            }
    }   
    
    Err(Error::msg("Could not connect to database after 3 attempts"))
}
