use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres};
use anyhow::Error;
use std::env;

pub async fn create_database() -> Result<(), Error> {
    let pg_host     = env::var("PG_HOST").expect("PG_HOST must be set");
    let pg_user     = env::var("PG_USER").expect("PG_USER must be set");
    let pg_password = env::var("PG_PASSWORD").expect("PG_PASSWORD must be set");
    let pg_database = env::var("PG_DATABASE").expect("PG_DATABASE must be set");
    
    let postgres_url = format!(
        "postgres://{}:{}@{}/postgres",
        pg_user, pg_password, pg_host
    );

    match PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_url)
        .await {
            Ok(pool) => {
                let query = format!("CREATE DATABASE {}", pg_database);
                match sqlx::query(&query).execute(&pool).await {
                    Ok(_)  => println!("Database created successfully"),
                    Err(e) => println!("Database already exist : {}", e ),
                }
                Ok(())
            },
            Err(e) => {
                println!("Could not connect to postgres database: {}", e);
                Err(Error::msg("Database connection failed"))
            }
    }
}

pub async fn init_db() -> Result<Pool<Postgres>, Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("Ensuring database exists...");

    if let Err(e) = create_database().await {
        println!("Warning: Could not create database: {}", e);
    }

    println!("Connecting to database...");
    
    for i in 1..=3 {
        match PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await {
                Ok(pool) => {
                    if let Ok(_) = sqlx::query("SELECT 1").execute(&pool).await {
                        if let Err(e) = create_tables(&pool).await {
                            println!("Warning: Could not create tables: {}", e);
                        }
                        println!("Database connection successful!");
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

async fn create_tables(pool: &PgPool) -> Result<(), Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE
        )"
    )
    .execute(pool)
    .await
    .map_err(|e| Error::msg(format!("Failed to create tables: {}", e)))?;

    Ok(())
}
