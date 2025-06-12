use sqlx::{query, PgPool, Row};
use anyhow::{Error, Result};
use uuid::Uuid;
use time::OffsetDateTime;
use crate::{repositories::_database_query::DatabaseQuery};
use std::collections::HashMap;

pub struct InitRepository {
    pool: PgPool,
}

impl InitRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialise la table des utilisateurs
    pub async fn init_users_table(&self) -> Result<()> {
        // Création table;
        DatabaseQuery::new(self.pool.clone()).run_query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
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
                )
        "#
        ).await?;

        // Création des indexes
        let indexes = vec!["login", "email", "created_at"];
        DatabaseQuery::new(self.pool.clone()).create_indexes("users", indexes).await?;

        // End
        println!("Database table initialized successfully");
        Ok(())
    }


    /// Initialise la table des migrations
    pub async fn init_migration_table(&self) -> Result<()> {
        DatabaseQuery::new(self.pool.clone()).run_query(
            r#"CREATE TABLE IF NOT EXISTS migration (
                id          SERIAL PRIMARY KEY NOT NULL,
                name        TEXT NOT NULL UNIQUE,
                description TEXT,
                created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )"#
        ).await?;

        // Création des indexes
        let indexes = vec!["created_at"];
        DatabaseQuery::new(self.pool.clone()).create_indexes("migration", indexes).await?;

        // End
        println!("Database migration initialized successfully");
        Ok(())
    }
}
