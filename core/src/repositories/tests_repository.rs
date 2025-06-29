use sqlx::Row;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use time::OffsetDateTime;
use uuid::Uuid;
use crate::repositories::_database::DatabaseQuery;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Tests {
    pub id: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

pub struct TestsRepository {
    db: DatabaseQuery,
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
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { db: DatabaseQuery::new(pool) }
    }

    /// Crée un nouvel enregistrement
    pub async fn create(&self, item: &Tests) -> Result<Tests> {
        let query = format!(
            "INSERT INTO tests (id, created_at, updated_at) VALUES ('{}', '{}', '{}')",
            item.id.unwrap_or(Uuid::new_v4()),
            item.created_at,
            item.updated_at
        );
        self.db.run_query(&query).await?;
        Ok(item.clone())
    }

    /// Recherche un enregistrement par ID
    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<Tests>> {
        let query = format!("SELECT * FROM tests WHERE id = '{}'", id);
        match self.db.run_query_fetch_optional(&query).await? {
            Some(row) => {
                let test = Tests {
                    id: row.get("id"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(Some(test))
            },
            None => Ok(None)
        }
    }

    /// Récupère tous les enregistrements
    pub async fn find_all(&self) -> Result<Vec<Tests>> {
        let query = "SELECT * FROM tests ORDER BY created_at DESC";
        let rows = self.db.run_query_fetch_all(query).await?;
        
        let mut tests_list = Vec::new();
        for row in rows {
            let test = Tests {
                id: row.get("id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            tests_list.push(test);
        }
        Ok(tests_list)
    }

    /// Met à jour un enregistrement
    pub async fn update(&self, item: &Tests) -> Result<Tests> {
        let now = OffsetDateTime::now_utc();
        let query = format!(
            "UPDATE tests SET updated_at = '{}' WHERE id = '{}'",
            now,
            item.id.unwrap_or(Uuid::new_v4())
        );
        self.db.run_query(&query).await?;
        Ok(item.clone())
    }

    /// Supprime un enregistrement
    pub async fn delete(&self, id: &Uuid) -> Result<bool> {
        let query = format!("DELETE FROM tests WHERE id = '{}'", id);
        match self.db.run_query(&query).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false)
        }
    }
}