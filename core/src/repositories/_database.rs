use sqlx::{query, Row, FromRow, Executor};
use anyhow::{Error, Result};
use uuid::Uuid;
use time::OffsetDateTime;
use std::collections::HashMap;
use std::env;
use crate::repositories::migration_repository::{MigrationRepository, Migration};
use crate::repositories::{_init_repository::InitRepository};
use crate::repositories::migrations;

#[cfg(feature = "postgres")]
use sqlx::{PgPool, postgres::PgPoolOptions, Pool, Postgres};

#[cfg(feature = "sqlite")]
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions, Sqlite};

/// Enum pour représenter les différents types de base de données
#[derive(Clone)]
pub enum DatabasePool {
    #[cfg(feature = "postgres")]
    Postgres(PgPool),
    #[cfg(feature = "sqlite")]
    Sqlite(SqlitePool),
}

/// Type alias pour les rows de base de données
pub enum DatabaseRow {
    #[cfg(feature = "postgres")]
    Postgres(sqlx::postgres::PgRow),
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::sqlite::SqliteRow),
}

impl DatabaseRow {
    /// Méthode helper pour accéder aux colonnes de manière générique  
    pub fn try_get<'r, T>(&'r self, column: &str) -> Result<T, sqlx::Error> 
    where
        T: sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + 'r,
        T: sqlx::Decode<'r, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + 'r,
    {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseRow::Postgres(row) => row.try_get(column),
            #[cfg(feature = "sqlite")]
            DatabaseRow::Sqlite(row) => row.try_get(column),
            #[cfg(not(any(feature = "postgres", feature = "sqlite")))]
            _ => unreachable!("No database feature enabled"),
        }
    }

    /// Méthode helper pour accéder aux colonnes de manière générique (version simplifiée)
    pub fn get<'r, T>(&'r self, column: &str) -> T 
    where
        T: sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + 'r,
        T: sqlx::Decode<'r, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + 'r,
    {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseRow::Postgres(row) => row.get(column),
            #[cfg(feature = "sqlite")]
            DatabaseRow::Sqlite(row) => row.get(column),
            #[cfg(not(any(feature = "postgres", feature = "sqlite")))]
            _ => unreachable!("No database feature enabled"),
        }
    }
}

/// Représente une requête de base de données avec adaptateur
#[derive(Clone)]
pub struct DatabaseQuery {
    pool: DatabasePool,
}

/// Implémentation de la structure DatabaseQuery
/// Cette structure encapsule un pool de connexions à la base de données
impl DatabaseQuery {
    #[cfg(feature = "postgres")]
    pub fn new_postgres(pool: PgPool) -> Self {
        Self { 
            pool: DatabasePool::Postgres(pool)
        }
    }
    
    #[cfg(feature = "sqlite")]
    pub fn new_sqlite(pool: SqlitePool) -> Self {
        Self { 
            pool: DatabasePool::Sqlite(pool)
        }
    }
    
    /// Crée une nouvelle instance de DatabaseQuery depuis un pool générique
    #[cfg(feature = "postgres")]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self::new_postgres(pool)
    }
    
    #[cfg(all(feature = "sqlite", not(feature = "postgres")))]
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self::new_sqlite(pool)
    }
    
    // Méthode pour récupérer une référence au pool de connexion
    pub fn get_pool(&self) -> &DatabasePool {
        &self.pool
    }   
    
    /// Lance une requête en SQL brut
    pub async fn run_query(&self, query: &str) -> Result<()> {
        match &self.pool {
            #[cfg(feature = "postgres")]
            DatabasePool::Postgres(pool) => {
                sqlx::query(query)
                    .execute(pool)
                    .await
                    .map_err(|e| Error::msg(format!("\x1b[31mFailed query {}\x1b[0m", e)))?;
            },
            #[cfg(feature = "sqlite")]
            DatabasePool::Sqlite(pool) => {
                sqlx::query(query)
                    .execute(pool)
                    .await
                    .map_err(|e| Error::msg(format!("\x1b[31mFailed query {}\x1b[0m", e)))?;
            },
            #[cfg(not(any(feature = "postgres", feature = "sqlite")))]
            _ => return Err(Error::msg("No database feature enabled")),
        }

        println!("\x1b[32mQuery executed successfully: {}\x1b[0m", query);
        Ok(())
    }
    
    /// Exécute une requête et retourne une ligne
    pub async fn run_query_fetch_one(&self, query: &str) -> Result<DatabaseRow> {
        let result = match &self.pool {
            #[cfg(feature = "postgres")]
            DatabasePool::Postgres(pool) => {
                let row = sqlx::query(query)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| Error::msg(format!("\x1b[31mFailed query_fetch_one: {}\x1b[0m", e)))?;
                DatabaseRow::Postgres(row)
            },
            #[cfg(feature = "sqlite")]
            DatabasePool::Sqlite(pool) => {
                let row = sqlx::query(query)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| Error::msg(format!("\x1b[31mFailed query_fetch_one: {}\x1b[0m", e)))?;
                DatabaseRow::Sqlite(row)
            },
            #[cfg(not(any(feature = "postgres", feature = "sqlite")))]
            _ => return Err(Error::msg("No database feature enabled")),
        };

        println!("\x1b[32mQuery fetch_one executed successfully: {}\x1b[0m", query);
        Ok(result)
    }
    
    /// Exécute une requête et retourne une ligne optionnelle
    pub async fn run_query_fetch_optional(&self, query: &str) -> Result<Option<DatabaseRow>> {
        let result = match &self.pool {
            #[cfg(feature = "postgres")]
            DatabasePool::Postgres(pool) => {
                match sqlx::query(query)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| Error::msg(format!("\x1b[31mFailed query_fetch_optional: {}\x1b[0m", e)))? {
                    Some(row) => Some(DatabaseRow::Postgres(row)),
                    None => None,
                }
            },
            #[cfg(feature = "sqlite")]
            DatabasePool::Sqlite(pool) => {
                match sqlx::query(query)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| Error::msg(format!("\x1b[31mFailed query_fetch_optional: {}\x1b[0m", e)))? {
                    Some(row) => Some(DatabaseRow::Sqlite(row)),
                    None => None,
                }
            },
            #[cfg(not(any(feature = "postgres", feature = "sqlite")))]
            _ => return Err(Error::msg("No database feature enabled")),
        };

        println!("\x1b[32mQuery fetch_optional executed successfully: {}\x1b[0m", query);
        Ok(result)
    }
    
    /// Exécute une requête et retourne toutes les lignes
    pub async fn run_query_fetch_all(&self, query: &str) -> Result<Vec<DatabaseRow>> {
        let result = match &self.pool {
            #[cfg(feature = "postgres")]
            DatabasePool::Postgres(pool) => {
                let rows = sqlx::query(query)
                    .fetch_all(pool)
                    .await
                    .map_err(|e| Error::msg(format!("\x1b[31mFailed query_fetch_all: {}\x1b[0m", e)))?;
                rows.into_iter().map(|row| DatabaseRow::Postgres(row)).collect()
            },
            #[cfg(feature = "sqlite")]
            DatabasePool::Sqlite(pool) => {
                let rows = sqlx::query(query)
                    .fetch_all(pool)
                    .await
                    .map_err(|e| Error::msg(format!("\x1b[31mFailed query_fetch_all: {}\x1b[0m", e)))?;
                rows.into_iter().map(|row| DatabaseRow::Sqlite(row)).collect()
            },
            #[cfg(not(any(feature = "postgres", feature = "sqlite")))]
            _ => return Err(Error::msg("No database feature enabled")),
        };

        println!("\x1b[32mQuery fetch_all executed successfully: {}\x1b[0m", query);
        Ok(result)
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
            self.run_query(&drop_index_query).await?;
        }
        
        println!("Indexes dropped successfully for table: {}", table_name);
        Ok(())
    }
    
    /// Vérifie si une base de données existe
    pub async fn database_exists(&self, database_name: &str) -> Result<bool> {
        match &self.pool {
            #[cfg(feature = "postgres")]
            DatabasePool::Postgres(pool) => {
                let query = "SELECT 1 FROM pg_database WHERE datname = $1";
                
                match sqlx::query(query)
                    .bind(database_name)
                    .fetch_optional(pool)
                    .await {
                        Ok(Some(_)) => Ok(true),
                        Ok(None) => Ok(false),
                        Err(e) => Err(Error::msg(format!("Failed to check database existence: {}", e))),
                    }
            },
            #[cfg(feature = "sqlite")]
            DatabasePool::Sqlite(_pool) => {
                // Pour SQLite, on considère que la base existe toujours
                // car elle est créée automatiquement lors de la connexion
                Ok(true)
            },
            #[cfg(not(any(feature = "postgres", feature = "sqlite")))]
            _ => Err(Error::msg("No database feature enabled")),
        }
    }
    
    /// Crée une base de données si elle n'existe pas
    pub async fn create_database_if_not_exists(&self, database_name: &str) -> Result<()> {
        match &self.pool {
            #[cfg(feature = "postgres")]
            DatabasePool::Postgres(_pool) => {
                if !self.database_exists(database_name).await? {
                    let create_query = format!("CREATE DATABASE \"{}\"", database_name);
                    self.run_query(&create_query).await?;
                    println!("\x1b[32mDatabase '{}' created successfully\x1b[0m", database_name);
                } else {
                    println!("\x1b[33mDatabase '{}' already exists\x1b[0m", database_name);
                }
            },
            #[cfg(feature = "sqlite")]
            DatabasePool::Sqlite(_pool) => {
                // Pour SQLite, la base de données est créée automatiquement lors de la connexion
                println!("\x1b[32mSQLite database '{}' is ready\x1b[0m", database_name);
            },
            #[cfg(not(any(feature = "postgres", feature = "sqlite")))]
            _ => return Err(Error::msg("No database feature enabled")),
        }
        Ok(())
    }
    
    /// Récupère le pool PostgreSQL sous-jacent (pour compatibilité avec l'ancien code)
    #[cfg(feature = "postgres")]
    pub fn get_postgres_pool(&self) -> Option<&sqlx::PgPool> {
        match &self.pool {
            DatabasePool::Postgres(pool) => Some(pool),
            #[cfg(feature = "sqlite")]
            DatabasePool::Sqlite(_) => None,
        }
    }
    
    /// Récupère le pool SQLite sous-jacent (pour compatibilité avec l'ancien code)
    #[cfg(feature = "sqlite")]
    pub fn get_sqlite_pool(&self) -> Option<&sqlx::SqlitePool> {
        match &self.pool {
            #[cfg(feature = "postgres")]
            DatabasePool::Postgres(_) => None,
            DatabasePool::Sqlite(pool) => Some(pool),
        }
    }
}


/// Exécute les migrations nécessaires pour créer les tables de base de données
pub async fn _init_migration_tables(db_query: &DatabaseQuery) {
    println!("Running migrations...");
    
    let mut failed_migrations = Vec::new();
    
    // Macro pour simplifier l'exécution des migrations
    macro_rules! run_migration {
        ($name:expr, $migration:expr) => {
            if let Err(err) = $migration.await {
                eprintln!("\x1b[31mMigration '{}' failed: {}\x1b[0m", $name, err);
                failed_migrations.push($name);
            }
        };
    }
    
    // Exécution séquentielle des migrations
    run_migration!("migration_create_migration", migrations::migration_create_migration::run(db_query));
    run_migration!("migration_create_users",     migrations::migration_create_users::run(db_query));
    run_migration!("migration_create_logs",      migrations::migration_create_logs::run(db_query));
    run_migration!("migration_test",             migrations::migration_test::run(db_query));
    
    // Affichage du résultat final
    if failed_migrations.is_empty() {
        println!("\x1b[32mAll migrations completed successfully.\x1b[0m");
    } else {
        eprintln!("\x1b[31m{} migration(s) failed: {}\x1b[0m", 
            failed_migrations.len(), 
            failed_migrations.join(", "));
    }
}


/// Initialise la base de données en créant un pool de connexions
pub async fn init_db() -> Result<DatabaseQuery, Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    
    // Détecter le type de base de données à partir de l'URL
    if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
        #[cfg(feature = "postgres")]
        {
            for i in 1..=3 {
                match PgPoolOptions::new()
                    .max_connections(5)
                    .connect(&database_url)
                    .await {
                        Ok(pool) => {
                            // Créer une instance de DatabaseQuery pour utiliser run_query
                            let db_query = DatabaseQuery::new_postgres(pool);

                            // Vérifier et créer la base de données si nécessaire
                            let database_name = env::var("PG_DATABASE").expect("PG_DATABASE must be set");
                            if let Err(e) = db_query.create_database_if_not_exists(&database_name).await {
                                println!("Warning: Could not create database (may already exist or insufficient permissions): {}", e);
                            }

                            if let Ok(_) = db_query.run_query("SELECT 1").await {
                                // Initialize and create tables if needed
                                _init_migration_tables(&db_query).await;

                                return Ok(db_query);
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
        }
        #[cfg(not(feature = "postgres"))]
        {
            return Err(Error::msg("PostgreSQL support not enabled (postgres feature not enabled)"));
        }
    } else if database_url.starts_with("sqlite://") {
        #[cfg(feature = "sqlite")]
        {
            for i in 1..=3 {
                match SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect(&database_url)
                    .await {
                        Ok(pool) => {
                            // Créer une instance de DatabaseQuery pour utiliser run_query
                            let db_query = DatabaseQuery::new_sqlite(pool);

                            // Pour SQLite, pas besoin de créer la base de données explicitement
                            if let Ok(_) = db_query.run_query("SELECT 1").await {
                                // Initialize and create tables if needed
                                _init_migration_tables(&db_query).await;

                                return Ok(db_query);
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
        }
        #[cfg(not(feature = "sqlite"))]
        {
            return Err(Error::msg("SQLite support not enabled (sqlite feature not enabled)"));
        }
    } else {
        return Err(Error::msg(format!("Unsupported database URL format: {}", database_url)));
    }
    
    Err(Error::msg("Could not connect to database after 3 attempts"))
}