use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(name: String, email: String) -> Self {
        let now = Utc::now();
        Self { 
            id: Uuid::new_v4(),
            name,
            email,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_name(&mut self, new_name: String) {
        self.name = new_name;
        self.updated_at = Utc::now();
    }

    pub fn update_email(&mut self, new_email: String) {
        self.email = new_email;
        self.updated_at = Utc::now();
    }

    /// Valide l'email avec une regex simple
    pub fn is_valid_email(&self) -> bool {
        self.email.contains('@') && self.email.contains('.')
    }
}
