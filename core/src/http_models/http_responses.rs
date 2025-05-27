use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct HttpSendResponse {
    pub status: u16,
    pub message: Option<String>,
    pub data: Option<Value>,
}
