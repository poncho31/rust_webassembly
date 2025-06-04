use sqlx::{PgPool, Row};
use anyhow::{Error, Result};
use uuid::Uuid;
use crate::db_models::{FormData, NewFormData};

pub struct FormRepository {
    pool: PgPool,
}

impl FormRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Créer la table form_data si elle n'existe pas
    pub async fn create_table_if_not_exists(&self) -> Result<()> {
        let query = r#"
            CREATE TABLE IF NOT EXISTS form_data (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                login TEXT,
                birthday DATE,
                firstname TEXT,
                lastname TEXT,
                sexe TEXT,
                age INTEGER,
                info TEXT,
                email TEXT,
                files_info JSONB,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#;

        sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::msg(format!("Failed to create form_data table: {}", e)))?;

        println!("✅ Table form_data créée/vérifiée avec succès");
        Ok(())
    }

    /// Insérer de nouvelles données de formulaire
    pub async fn insert_form_data(&self, form_data: NewFormData) -> Result<Uuid> {
        let files_json = form_data.files_info
            .as_ref()
            .map(|files| serde_json::to_value(files).unwrap_or(serde_json::Value::Null))
            .unwrap_or(serde_json::Value::Null);

        let query = r#"
            INSERT INTO form_data (login, birthday, firstname, lastname, sexe, age, info, email, files_info)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
        "#;

        let row = sqlx::query(query)
            .bind(&form_data.login)
            .bind(&form_data.birthday)
            .bind(&form_data.firstname)
            .bind(&form_data.lastname)
            .bind(&form_data.sexe)
            .bind(&form_data.age)
            .bind(&form_data.info)
            .bind(&form_data.email)
            .bind(&files_json)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::msg(format!("Failed to insert form data: {}", e)))?;

        let id: Uuid = row.get("id");
        println!("✅ Données de formulaire sauvegardées avec l'ID: {}", id);
        Ok(id)
    }

    /// Récupérer toutes les données de formulaire
    pub async fn get_all_form_data(&self) -> Result<Vec<FormData>> {
        let query = r#"
            SELECT id, login, birthday, firstname, lastname, sexe, age, info, email, 
                   files_info, created_at
            FROM form_data 
            ORDER BY created_at DESC
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::msg(format!("Failed to fetch form data: {}", e)))?;

        let mut form_data_list = Vec::new();
        for row in rows {
            let files_info: Option<Vec<String>> = row.get::<Option<serde_json::Value>, _>("files_info")
                .and_then(|v| serde_json::from_value(v).ok());

            let form_data = FormData {
                id: row.get("id"),
                login: row.get("login"),
                birthday: row.get("birthday"),
                firstname: row.get("firstname"),
                lastname: row.get("lastname"),
                sexe: row.get("sexe"),
                age: row.get("age"),
                info: row.get("info"),
                email: row.get("email"),
                files_info,
                created_at: row.get("created_at"),
            };
            form_data_list.push(form_data);
        }

        Ok(form_data_list)
    }

    /// Récupérer une donnée de formulaire par ID
    pub async fn get_form_data_by_id(&self, id: Uuid) -> Result<Option<FormData>> {
        let query = r#"
            SELECT id, login, birthday, firstname, lastname, sexe, age, info, email, 
                   files_info, created_at
            FROM form_data 
            WHERE id = $1
        "#;

        let row = sqlx::query(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::msg(format!("Failed to fetch form data by id: {}", e)))?;

        if let Some(row) = row {
            let files_info: Option<Vec<String>> = row.get::<Option<serde_json::Value>, _>("files_info")
                .and_then(|v| serde_json::from_value(v).ok());

            let form_data = FormData {
                id: row.get("id"),
                login: row.get("login"),
                birthday: row.get("birthday"),
                firstname: row.get("firstname"),
                lastname: row.get("lastname"),
                sexe: row.get("sexe"),
                age: row.get("age"),
                info: row.get("info"),
                email: row.get("email"),
                files_info,
                created_at: row.get("created_at"),
            };
            Ok(Some(form_data))
        } else {
            Ok(None)
        }
    }

    /// Supprimer une donnée de formulaire par ID
    pub async fn delete_form_data(&self, id: Uuid) -> Result<bool> {
        let query = "DELETE FROM form_data WHERE id = $1";

        let result = sqlx::query(query)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::msg(format!("Failed to delete form data: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// Compter le nombre total d'enregistrements
    pub async fn count_form_data(&self) -> Result<i64> {
        let query = "SELECT COUNT(*) as count FROM form_data";

        let row = sqlx::query(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::msg(format!("Failed to count form data: {}", e)))?;

        Ok(row.get("count"))
    }
}
