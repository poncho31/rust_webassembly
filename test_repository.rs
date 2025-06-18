use sqlx::{PgPool, Row, FromRow};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, FromRow)]
pub struct Test {
    pub id: Option<Uuid>,
    pub r#type: String,
    pub level: i32,
    pub message: String,
    pub context: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

pub struct TestRepository {
    pool: PgPool,
}

impl Test {
    pub fn new(r#type: &str,
        level: &i32,
        message: &str,
        context: Option<&str>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Uuid::new_v4(),
                    r#type: r#type.to_string(),
                    level: *level,
                    message: message.to_string(),
                    context: context.map(|s| s.to_string()),
            created_at: now,
            updated_at: now,
        }
    }
}

impl TestRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Crée un nouvel enregistrement
    pub async fn create(&self, item: &Test) -> Result<Test> {
        let result = sqlx::query_as::<_, Test>(
            r#"INSERT INTO test (id, \"type\", level, message, context, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"#
        )
        .bind(&item.id)
        .bind(&item.r#type)
        .bind(&item.level)
        .bind(&item.message)
        .bind(&item.context)
        .bind(&item.created_at)
        .bind(&item.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Recherche un enregistrement par ID
    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<Test>> {
        let result = sqlx::query_as::<_, Test>(
            r#"SELECT * FROM test WHERE id = $1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Récupère tous les enregistrements
    pub async fn find_all(&self) -> Result<Vec<Test>> {
        let results = sqlx::query_as::<_, Test>(
            r#"SELECT * FROM test ORDER BY created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Met à jour un enregistrement
    pub async fn update(&self, item: &Test) -> Result<Test> {
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query_as::<_, Test>(
            r#"UPDATE test SET \"type\" = $2, level = $3, message = $4, context = $5, updated_at = $7, updated_at = $8"#
            .to_string() + " WHERE id = $1 RETURNING *"
        )
        .bind(&item.id)
        .bind(&item.r#type)
        .bind(&item.level)
        .bind(&item.message)
        .bind(&item.context)
        .bind(&item.updated_at)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Supprime un enregistrement
    pub async fn delete(&self, id: &Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM test WHERE id = $1"#
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}