use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use anyhow::{Error, Result};
use std::env;

/// Crée un pool de connexions SQLite
pub async fn create_sqlite_pool() -> Result<Pool<Sqlite>, Error> {
    let database_url = env::var("DATABASE_URL_SQLITE")
        .unwrap_or_else(|_| "sqlite://storage/database/rust_webassembly.sqlite".to_string());

    println!("Connecting to SQLite database: {}", database_url);
    
    // Créer le répertoire s'il n'existe pas
    if let Some(parent) = std::path::Path::new(&database_url.replace("sqlite://", "")).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::msg(format!("Failed to create SQLite directory: {}", e)))?;
    }
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| Error::msg(format!("Failed to connect to SQLite: {}", e)))?;

    // Test de connexion et activation des clés étrangères
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .map_err(|e| Error::msg(format!("SQLite pragma setup failed: {}", e)))?;

    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| Error::msg(format!("SQLite connection test failed: {}", e)))?;

    println!("\x1b[32mSQLite connection established successfully\x1b[0m");
    Ok(pool)
}

/// Vérifie si une table SQLite existe
pub async fn table_exists(pool: &Pool<Sqlite>, table_name: &str) -> Result<bool> {
    let query = "SELECT name FROM sqlite_master WHERE type='table' AND name = ?";
    
    match sqlx::query(query)
        .bind(table_name)
        .fetch_optional(pool)
        .await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(Error::msg(format!("Failed to check SQLite table existence: {}", e))),
        }
}

/// Configure SQLite avec les paramètres optimaux
pub async fn configure_sqlite(pool: &Pool<Sqlite>) -> Result<()> {
    let pragmas = vec![
        "PRAGMA foreign_keys = ON",
        "PRAGMA journal_mode = WAL",
        "PRAGMA synchronous = NORMAL",
        "PRAGMA cache_size = 1000",
        "PRAGMA temp_store = MEMORY",
    ];

    for pragma in pragmas {
        sqlx::query(pragma)
            .execute(pool)
            .await
            .map_err(|e| Error::msg(format!("Failed to set SQLite pragma: {}", e)))?;
    }

    println!("\x1b[32mSQLite configuration completed\x1b[0m");
    Ok(())
}
