use sqlx::Row;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use time::OffsetDateTime;
use crate::repositories::_database::DatabaseQuery;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Migration {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

pub struct MigrationRepository {
    db: DatabaseQuery,
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
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { db: DatabaseQuery::new(pool) }
    }

    /// Recherche un enregistrement par nom
    pub async fn find_by_name(&self, name: &str) -> Result<Option<Migration>> {
        let query = format!("SELECT * FROM migration WHERE name = '{}'", name);
        match self.db.run_query_fetch_optional(&query).await? {
            Some(row) => {
                let migration = Migration {
                    id: row.get("id"),
                    name: row.get("name"),
                    description: row.get("description"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(Some(migration))
            },
            None => Ok(None)
        }
    }

    /// Crée un nouvel enregistrement
    pub async fn create(&self, item: &Migration) -> Result<Migration> {
        let query = format!(
            "INSERT INTO migration (name, description, created_at, updated_at) VALUES ('{}', '{}', '{}', '{}')",
            item.name,
            item.description.as_ref().unwrap_or(&"".to_string()),
            item.created_at,
            item.updated_at
        );
        self.db.run_query(&query).await?;
        Ok(item.clone())
    }

    /// Recherche un enregistrement par ID
    pub async fn find_by_id(&self, id: i32) -> Result<Option<Migration>> {
        let query = format!("SELECT * FROM migration WHERE id = {}", id);
        match self.db.run_query_fetch_optional(&query).await? {
            Some(row) => {
                let migration = Migration {
                    id: row.get("id"),
                    name: row.get("name"),
                    description: row.get("description"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(Some(migration))
            },
            None => Ok(None)
        }
    }

    /// Récupère tous les enregistrements
    pub async fn find_all(&self) -> Result<Vec<Migration>> {
        let query = "SELECT * FROM migration ORDER BY created_at DESC";
        let rows = self.db.run_query_fetch_all(query).await?;
        
        let mut migrations = Vec::new();
        for row in rows {
            let migration = Migration {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            migrations.push(migration);
        }
        Ok(migrations)
    }

    /// Met à jour un enregistrement
    pub async fn update(&self, item: &Migration) -> Result<Migration> {
        let now = OffsetDateTime::now_utc();
        let query = format!(
            "UPDATE migration SET name = '{}', description = '{}', updated_at = '{}' WHERE id = {}",
            item.name,
            item.description.as_ref().unwrap_or(&"".to_string()),
            now,
            item.id
        );
        self.db.run_query(&query).await?;
        Ok(item.clone())
    }

    /// Supprime un enregistrement
    pub async fn delete(&self, id: i32) -> Result<bool> {
        let query = format!("DELETE FROM migration WHERE id = {}", id);
        match self.db.run_query(&query).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false)
        }
    }
}