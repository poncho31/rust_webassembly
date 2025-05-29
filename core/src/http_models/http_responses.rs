use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HttpSendResponse {
    pub status  : u16,
    pub message : Option<String>,
    pub data    : Option<Value>,
}

impl HttpSendResponse {
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    pub fn get_data<T>(&self) -> Option<T> 
    where T: for<'a> Deserialize<'a> {
        self.data.as_ref().and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| "No message".to_string())
    }
}
