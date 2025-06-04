use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FormData {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFormData {
    pub login: Option<String>,
    pub birthday: Option<String>,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub sexe: Option<String>,
    pub age: Option<i32>,
    pub info: Option<String>,
    pub email: Option<String>,
    pub files_info: Option<String>,
}

impl NewFormData {
    pub fn from_form_fields(
        fields: &HashMap<String, String>,
        files: &Vec<String>,
    ) -> Self {
        let birthday = fields.get("birthday").cloned();
        let age = fields.get("age")
            .and_then(|age_str| age_str.parse::<i32>().ok());
        
        Self {
            login: fields.get("login").cloned(),
            birthday,
            firstname: fields.get("firstname").cloned(),
            lastname: fields.get("lastname").cloned(),
            sexe: fields.get("sexe").cloned(),
            age,
            info: fields.get("info").cloned(),
            email: fields.get("email").cloned(),
            files_info: if files.is_empty() { None } else { Some(files.join(",")) },
        }
    }
}
