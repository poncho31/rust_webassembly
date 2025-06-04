use sqlx::{PgPool, Row};
use anyhow::{Error, Result};
use uuid::Uuid;
use time::OffsetDateTime;
use crate::db_models::form_data::{FormData, NewFormData};
use std::collections::HashMap;

pub struct DatabaseRepository {
    pool: PgPool,
}

impl DatabaseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Vérifie si un utilisateur existe déjà par login
    pub async fn user_exists_by_login(&self, login: &str) -> Result<bool> {
        let result = sqlx::query(
            "SELECT COUNT(*) as count FROM form_data WHERE login = $1"
        )
        .bind(login)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = result.get("count");
        Ok(count > 0)
    }

    /// Récupère un utilisateur par login
    pub async fn get_user_by_login(&self, login: &str) -> Result<Option<FormData>> {
        let result = sqlx::query_as::<_, FormData>(
            "SELECT * FROM form_data WHERE login = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(login)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Crée un nouvel utilisateur
    pub async fn create_user(&self, new_form_data: &NewFormData) -> Result<FormData> {
        let id = Uuid::new_v4();
        let created_at = OffsetDateTime::now_utc();

        let form_data = sqlx::query_as::<_, FormData>(
            "INSERT INTO form_data (id, login, birthday, firstname, lastname, sexe, age, info, email, files_info, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING *"
        )
        .bind(id)
        .bind(&new_form_data.login)
        .bind(&new_form_data.birthday)
        .bind(&new_form_data.firstname)
        .bind(&new_form_data.lastname)
        .bind(&new_form_data.sexe)
        .bind(new_form_data.age)
        .bind(&new_form_data.info)
        .bind(&new_form_data.email)
        .bind(&new_form_data.files_info)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(form_data)
    }

    /// Met à jour un utilisateur existant
    pub async fn update_user(&self, login: &str, new_form_data: &NewFormData) -> Result<FormData> {
        let form_data = sqlx::query_as::<_, FormData>(
            "UPDATE form_data 
             SET birthday = $2, firstname = $3, lastname = $4, sexe = $5, age = $6, 
                 info = $7, email = $8, files_info = $9, created_at = $10
             WHERE login = $1
             RETURNING *"
        )
        .bind(login)
        .bind(&new_form_data.birthday)
        .bind(&new_form_data.firstname)
        .bind(&new_form_data.lastname)
        .bind(&new_form_data.sexe)
        .bind(new_form_data.age)
        .bind(&new_form_data.info)
        .bind(&new_form_data.email)
        .bind(&new_form_data.files_info)
        .bind(OffsetDateTime::now_utc())
        .fetch_one(&self.pool)
        .await?;

        Ok(form_data)
    }

    /// Crée ou met à jour un utilisateur (méthode principale)
    pub async fn upsert_user(&self, form_fields: &HashMap<String, String>, files: &Vec<String>) -> Result<FormData> {
        let new_form_data = NewFormData::from_form_fields(form_fields, files);

        // Vérifier si l'utilisateur a un login
        let login = match &new_form_data.login {
            Some(l) if !l.is_empty() => l,
            _ => return Err(Error::msg("Login is required")),
        };

        // Vérifier si l'utilisateur existe déjà
        if self.user_exists_by_login(login).await? {
            println!("User with login '{}' exists, updating...", login);
            self.update_user(login, &new_form_data).await
        } else {
            println!("Creating new user with login '{}'...", login);
            self.create_user(&new_form_data).await
        }
    }

    /// Initialise les tables nécessaires
    pub async fn init_tables(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS form_data (
                id UUID PRIMARY KEY,
                login TEXT,
                birthday TEXT,
                firstname TEXT,
                lastname TEXT,
                sexe TEXT,
                age INTEGER,
                info TEXT,
                email TEXT,
                files_info TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::msg(format!("Failed to create form_data table: {}", e)))?;

        // Créer un index sur login pour les recherches rapides
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_form_data_login ON form_data (login)"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::msg(format!("Failed to create index: {}", e)))?;

        println!("Database tables initialized successfully");
        Ok(())
    }
}
