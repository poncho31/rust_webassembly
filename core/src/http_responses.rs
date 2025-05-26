use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct HttpSendResponse {
    pub status: String,
    pub message: String,
}
