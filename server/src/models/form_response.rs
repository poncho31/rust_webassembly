use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize)]
pub struct FormResponse {
    pub form_fields: HashMap<String, String>,
    pub files: Vec<String>,
}
