use sqlx::{PgPool, Row};
use anyhow::{Error, Result};
use time::OffsetDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Migration {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewMigration {
    pub name: String,
    pub description: Option<String>,
}

pub struct MigrationRepository {
    pool: PgPool,
}

impl MigrationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Crée une nouvelle migration
    pub async fn create_migration(&self, new_migration: &NewMigration) -> Result<Migration> {
        let migration = sqlx::query_as::<_, Migration>(
            "INSERT INTO migration (name, description, created_at, updated_at)
             VALUES ($1, $2, NOW(), NOW())
             RETURNING *"
        )
        .bind(&new_migration.name)
        .bind(&new_migration.description)
        .fetch_one(&self.pool)
        .await?;

        Ok(migration)
    }

    /// Récupère toutes les migrations
    pub async fn get_all_migrations(&self) -> Result<Vec<Migration>> {
        let migrations = sqlx::query_as::<_, Migration>("SELECT * FROM migration ORDER BY created_at ASC")
            .fetch_all(&self.pool)
            .await?;
        
        Ok(migrations)
    }

    /// Récupère la dernière migration
    pub async fn get_latest_migration(&self) -> Result<Option<Migration>> {
        let migration = sqlx::query_as::<_, Migration>("SELECT * FROM migration ORDER BY created_at DESC LIMIT 1")
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(migration)
    }
}
