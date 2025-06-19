use sqlx::{PgPool, Row, FromRow};
use serde::{Serialize, Deserialize};
use anyhow::{Error, Result};
use time::OffsetDateTime;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::{repositories::_database::DatabaseQuery};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, FromRow)]
pub struct User {
    pub id: Uuid,
    pub login: Option<String>,
    pub birthday: Option<String>,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub sexe: Option<String>,
    pub age: Option<i32>,
    pub info: Option<String>,
    pub email: Option<String>,
    pub files_info: Option<String>,
    pub created_at: OffsetDateTime,
}

pub struct UserRepository {
    pool: PgPool,
}

impl User {
    pub fn from_form_fields(
        fields: &HashMap<String, String>,
        files: &Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            
            login: fields.get("login").cloned(),      // Utilisation de `cloned()` pour obtenir une Option<String> à partir de &String
            birthday : fields.get("birthday").cloned(),
            firstname: fields.get("firstname").cloned(),
            lastname: fields.get("lastname").cloned(),
            sexe: fields.get("sexe").cloned(),
            age : fields.get("age").and_then(|age_str| age_str.parse::<i32>().ok()),
            info: fields.get("info").cloned(),
            email: fields.get("email").cloned(),
            files_info: if files.is_empty() { None } else { Some(files.join(",")) },
            created_at: OffsetDateTime::now_utc(),
        }
    }
}




impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Vérifie si un utilisateur existe déjà par login
    pub async fn user_exists(&self, login: &str) -> Result<bool> {
        let result = sqlx::query(
            "SELECT COUNT(*) as count FROM users WHERE login = $1"
        )
        .bind(login)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = result.get("count");
        Ok(count > 0)
    }

    /// Récupère un utilisateur par login
    pub async fn get_user(&self, login: &str) -> Result<Option<User>> {
        let result = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE login = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(login)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Crée un nouvel utilisateur
    pub async fn create_user(&self, user: &User) -> Result<User> {
        let id = Uuid::new_v4();
        let created_at = OffsetDateTime::now_utc();

        let users = sqlx::query_as::<_, User>(
            "INSERT INTO users 
                    (id, login, birthday, firstname, lastname, sexe, age, info, email, files_info, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING *"
        )
        .bind(id)
        .bind(&user.login)
        .bind(&user.birthday)
        .bind(&user.firstname)
        .bind(&user.lastname)
        .bind(&user.sexe)
        .bind(&user.age)
        .bind(&user.info)
        .bind(&user.email)
        .bind(&user.files_info)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(users)
    }

    /// Met à jour un utilisateur existant
    pub async fn update_user(&self, login: &str, user: &User) -> Result<User> {
        let users = sqlx::query_as::<_, User>(
            "UPDATE users 
             SET birthday = $2, firstname = $3, lastname = $4, sexe = $5, age = $6, 
                 info = $7, email = $8, files_info = $9, created_at = $10
             WHERE login = $1
             RETURNING *"
        )
        .bind(login)
        .bind(&user.birthday)
        .bind(&user.firstname)
        .bind(&user.lastname)
        .bind(&user.sexe)
        .bind(&user.age)
        .bind(&user.info)
        .bind(&user.email)
        .bind(&user.files_info)
        .bind(OffsetDateTime::now_utc())
        .fetch_one(&self.pool)
        .await?;

        Ok(users)
    }

    /// Crée ou met à jour un utilisateur (méthode principale)
    pub async fn upsert_user(&self, form_fields: &HashMap<String, String>, files: &Vec<String>) -> Result<User> {
        let user = User::from_form_fields(form_fields, files);

        // Vérifier si l'utilisateur a un login
        let login = match &user.login {
            Some(l) if !l.is_empty() => l,
            _ => return Err(Error::msg("Login is required")),
        };

        // Vérifier si l'utilisateur existe déjà
        if self.user_exists(login).await? {
            println!("User with login '{}' exists, updating...", login);
            self.update_user(login, &user).await
        } else {
            println!("Creating new user with login '{}'...", login);
            self.create_user(&user).await
        }
    }


    /// Récupérer toutes les données de formulaire
    pub async fn get_all(&self) -> Result<Vec<User>> {
        let query = r#"
            SELECT id, login, birthday, firstname, lastname, sexe, age, info, email, 
                   files_info, created_at
            FROM users 
            ORDER BY created_at DESC
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::msg(format!("Failed to fetch form data: {}", e)))?;        
        
        let mut users_list = Vec::new();
        for row in rows {
            let files_info: Option<String> = row.get("files_info");

            let users = User {
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
            users_list.push(users);
        }

        Ok(users_list)
    }

    /// Supprimer une donnée de formulaire par ID
    pub async fn delete_user(&self, id: Uuid) -> Result<bool> {
        let query = "DELETE FROM users WHERE id = $1";

        let result = sqlx::query(query)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::msg(format!("Failed to delete form data: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }
}
