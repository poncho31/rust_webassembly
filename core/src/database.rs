use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use anyhow::Error;
use std::env;
use crate::repositories::{_init_repository::InitRepository};


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

                        // USER
                        println!("Initializing user tables...");
                        if let Err(e) = InitRepository::new(pool.clone()).init_users_table().await {
                            println!("Warning: Could not create user tables: {}", e);
                        }
                        println!("Database user successfully initialized");

                        // MIGRATION
                        println!("Initializing migration tables...");
                        if let Err(e) = InitRepository::new(pool.clone()).init_migration_table().await {
                            println!("Warning: Could not create migration tables: {}", e);
                        }
                        println!("Database migration successfully initialized");

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
