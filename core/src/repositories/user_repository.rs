use sqlx::Row;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use time::OffsetDateTime;
use std::collections::HashMap;
use uuid::Uuid;
use crate::repositories::_database::DatabaseQuery;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
    db: DatabaseQuery,
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
    pub fn new(db_query: DatabaseQuery) -> Self {
        Self { db: db_query }
    }

    /// Vérifie si un utilisateur existe déjà par login
    pub async fn user_exists(&self, login: &str) -> Result<bool> {
        let query = format!("SELECT COUNT(*) as count FROM users WHERE login = '{}'", login);
        let row = self.db.run_query_fetch_one(&query).await?;
        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    /// Récupère un utilisateur par login
    pub async fn get_user(&self, login: &str) -> Result<Option<User>> {
        let query = format!("SELECT * FROM users WHERE login = '{}' ORDER BY created_at DESC LIMIT 1", login);
        match self.db.run_query_fetch_optional(&query).await? {
            Some(row) => {
                let user = User {
                    id: row.get("id"),
                    login: row.get("login"),
                    birthday: row.get("birthday"),
                    firstname: row.get("firstname"),
                    lastname: row.get("lastname"),
                    sexe: row.get("sexe"),
                    age: row.get("age"),
                    info: row.get("info"),
                    email: row.get("email"),
                    files_info: row.get("files_info"),
                    created_at: row.get("created_at"),
                };
                Ok(Some(user))
            },
            None => Ok(None)
        }
    }

    /// Crée un nouvel utilisateur
    pub async fn create_user(&self, user: &User) -> Result<User> {
        let query = format!(
            "INSERT INTO users (id, login, birthday, firstname, lastname, sexe, age, info, email, files_info, created_at) VALUES ('{}', '{}', '{}', '{}', '{}', '{}', {}, '{}', '{}', '{}', '{}')",
            user.id,
            user.login.as_ref().unwrap_or(&"".to_string()),
            user.birthday.as_ref().unwrap_or(&"".to_string()),
            user.firstname.as_ref().unwrap_or(&"".to_string()),
            user.lastname.as_ref().unwrap_or(&"".to_string()),
            user.sexe.as_ref().unwrap_or(&"".to_string()),
            user.age.unwrap_or(0),
            user.info.as_ref().unwrap_or(&"".to_string()),
            user.email.as_ref().unwrap_or(&"".to_string()),
            user.files_info.as_ref().unwrap_or(&"".to_string()),
            user.created_at
        );

        self.db.run_query(&query).await?;
        Ok(user.clone())
    }

    /// Met à jour un utilisateur existant
    pub async fn update_user(&self, login: &str, user: &User) -> Result<User> {
        let query = format!(
            "UPDATE users SET birthday = '{}', firstname = '{}', lastname = '{}', sexe = '{}', age = {}, info = '{}', email = '{}', files_info = '{}', created_at = '{}' WHERE login = '{}'",
            user.birthday.as_ref().unwrap_or(&"".to_string()),
            user.firstname.as_ref().unwrap_or(&"".to_string()),
            user.lastname.as_ref().unwrap_or(&"".to_string()),
            user.sexe.as_ref().unwrap_or(&"".to_string()),
            user.age.unwrap_or(0),
            user.info.as_ref().unwrap_or(&"".to_string()),
            user.email.as_ref().unwrap_or(&"".to_string()),
            user.files_info.as_ref().unwrap_or(&"".to_string()),
            OffsetDateTime::now_utc(),
            login
        );

        self.db.run_query(&query).await?;
        Ok(user.clone())
    }

    /// Crée ou met à jour un utilisateur (méthode principale)
    pub async fn upsert_user(&self, form_fields: &HashMap<String, String>, files: &Vec<String>) -> Result<User> {
        let user = User::from_form_fields(form_fields, files);

        // Vérifier si l'utilisateur a un login
        let login = match &user.login {
            Some(l) if !l.is_empty() => l,
            _ => return Err(anyhow::Error::msg("Login is required")),
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
        let query = "SELECT id, login, birthday, firstname, lastname, sexe, age, info, email, files_info, created_at FROM users ORDER BY created_at DESC";

        let rows = self.db.run_query_fetch_all(query).await?;
        
        let mut users_list = Vec::new();
        for row in rows {
            let user = User {
                id: row.get("id"),
                login: row.get("login"),
                birthday: row.get("birthday"),
                firstname: row.get("firstname"),
                lastname: row.get("lastname"),
                sexe: row.get("sexe"),
                age: row.get("age"),
                info: row.get("info"),
                email: row.get("email"),
                files_info: row.get("files_info"),
                created_at: row.get("created_at"),
            };
            users_list.push(user);
        }

        Ok(users_list)
    }

    /// Supprimer une donnée de formulaire par ID
    pub async fn delete_user(&self, id: Uuid) -> Result<bool> {
        let query = format!("DELETE FROM users WHERE id = '{}'", id);
        match self.db.run_query(&query).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false)
        }
    }
}
