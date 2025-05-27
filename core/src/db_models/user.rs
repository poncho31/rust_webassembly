use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: u16,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(id: u16, name: String, email: String) -> Self {
        Self { id, name, email }
    }
}
