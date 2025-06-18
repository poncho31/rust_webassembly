use sqlx::{pool, postgres::PgPoolOptions, Pool, Postgres};
use anyhow::Error;
use std::env;
use crate::repositories::{_init_repository::InitRepository};
use crate::repositories::migrations;
use crate::repositories::_database_query::DatabaseQuery;

async fn create_tables(pool: &Pool<Postgres>) {
    // USER-
    if let Err(e) = InitRepository::new(pool.clone()).init_users_table().await {
        println!("Warning: Could not create user tables: {}", e);
    }

    // MIGRATION
    if let Err(e) = InitRepository::new(pool.clone()).init_migration_table().await {
        println!("Warning: Could not create migration tables: {}", e);
    }

    // Exécution des migrations via le module migrations
    println!("Running migrations...");
    let repo: DatabaseQuery = DatabaseQuery::new(pool.clone());
    
    if let Err(e) = migrations::migration_create_logs::migrate(&repo).await {
        println!("Warning: Migration 001 failed: {}", e);
        if let Err(e) = migrations::migration_create_logs::rollback(&repo).await {
            println!("Warning: Rollback failed: {}", e);
        }
    }
    
}

pub async fn init_db() -> Result<Pool<Postgres>, Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    
    for i in 1..=3 {            
        match PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await {
                Ok(pool) => {
                    if let Ok(_) = sqlx::query("SELECT 1").execute(&pool).await {
                        // Initialize and create tables if needed
                        create_tables(&pool).await;

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
    }    Err(Error::msg("Could not connect to database after 3 attempts"))
}
