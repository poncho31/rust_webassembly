use sqlx::{PgPool, Row, FromRow};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, FromRow)]
pub struct Migration {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

pub struct MigrationRepository {
    pool: PgPool,
}

impl Migration {
    pub fn new(name: &str, description: Option<&str>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: 0, // La valeur sera générée par PostgreSQL avec SERIAL
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            created_at: now,
            updated_at: now,
        }
    }
}

impl MigrationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Recherche un enregistrement par nom
    pub async fn find_by_name(&self, name: &str) -> Result<Option<Migration>> {
        let result = sqlx::query_as::<_, Migration>(
            r#"SELECT * FROM migration WHERE name = $1"#
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Crée un nouvel enregistrement
    pub async fn create(&self, item: &Migration) -> Result<Migration> {
        let result = sqlx::query_as::<_, Migration>(
            r#"INSERT INTO migration (name, description, created_at, updated_at) VALUES ($1, $2, $3, $4) RETURNING *"#
        )
        .bind(&item.name)
        .bind(&item.description)
        .bind(&item.created_at)
        .bind(&item.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Recherche un enregistrement par ID
    pub async fn find_by_id(&self, id: i32) -> Result<Option<Migration>> {
        let result = sqlx::query_as::<_, Migration>(
            r#"SELECT * FROM migration WHERE id = $1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Récupère tous les enregistrements
    pub async fn find_all(&self) -> Result<Vec<Migration>> {
        let results = sqlx::query_as::<_, Migration>(
            r#"SELECT * FROM migration ORDER BY created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Met à jour un enregistrement
    pub async fn update(&self, item: &Migration) -> Result<Migration> {
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query_as::<_, Migration>(
            r#"UPDATE migration SET name = $2, description = $3, updated_at = $4 WHERE id = $1 RETURNING *"#
        )
        .bind(item.id)
        .bind(&item.name)
        .bind(&item.description)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Supprime un enregistrement
    pub async fn delete(&self, id: i32) -> Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM migration WHERE id = $1"#
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}