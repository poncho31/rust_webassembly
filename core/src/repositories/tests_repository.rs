use sqlx::{PgPool, Row, FromRow};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, FromRow)]
pub struct Tests {
    pub id: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

pub struct TestsRepository {
    pool: PgPool,
}

impl Tests {
    pub fn new() -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl TestsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Crée un nouvel enregistrement
    pub async fn create(&self, item: &Tests) -> Result<Tests> {
        let result = sqlx::query_as::<_, Tests>(
            r#"INSERT INTO tests (id, created_at, updated_at) VALUES ($1, $2, $3) RETURNING *"#
        )
        .bind(&item.id)
        .bind(&item.created_at)
        .bind(&item.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Recherche un enregistrement par ID
    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<Tests>> {
        let result = sqlx::query_as::<_, Tests>(
            r#"SELECT * FROM tests WHERE id = $1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Récupère tous les enregistrements
    pub async fn find_all(&self) -> Result<Vec<Tests>> {
        let results = sqlx::query_as::<_, Tests>(
            r#"SELECT * FROM tests ORDER BY created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Met à jour un enregistrement
    pub async fn update(&self, item: &Tests) -> Result<Tests> {
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query_as::<_, Tests>(
            r#"UPDATE tests SET updated_at = $3, updated_at = $4"#
            .to_string() + " WHERE id = $1 RETURNING *"
        )
        .bind(&item.id)
        .bind(&item.updated_at)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Supprime un enregistrement
    pub async fn delete(&self, id: &Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM tests WHERE id = $1"#
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}