use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use anyhow::{Error, Result};
use std::env;

/// Crée un pool de connexions PostgreSQL
pub async fn create_postgres_pool() -> Result<Pool<Postgres>, Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to PostgreSQL database...");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| Error::msg(format!("Failed to connect to PostgreSQL: {}", e)))?;

    // Test de connexion
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| Error::msg(format!("PostgreSQL connection test failed: {}", e)))?;

    println!("\x1b[32mPostgreSQL connection established successfully\x1b[0m");
    Ok(pool)
}

/// Vérifie si une base de données PostgreSQL existe
pub async fn database_exists(pool: &Pool<Postgres>, database_name: &str) -> Result<bool> {
    let query = "SELECT 1 FROM pg_database WHERE datname = $1";
    
    match sqlx::query(query)
        .bind(database_name)
        .fetch_optional(pool)
        .await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(Error::msg(format!("Failed to check PostgreSQL database existence: {}", e))),
        }
}

/// Crée une base de données PostgreSQL si elle n'existe pas
pub async fn create_database_if_not_exists(pool: &Pool<Postgres>, database_name: &str) -> Result<()> {
    if !database_exists(pool, database_name).await? {
        let create_query = format!("CREATE DATABASE \"{}\"", database_name);
        sqlx::query(&create_query)
            .execute(pool)
            .await
            .map_err(|e| Error::msg(format!("Failed to create PostgreSQL database: {}", e)))?;
        println!("\x1b[32mPostgreSQL database '{}' created successfully\x1b[0m", database_name);
    } else {
        println!("\x1b[33mPostgreSQL database '{}' already exists\x1b[0m", database_name);
    }
    Ok(())
}
