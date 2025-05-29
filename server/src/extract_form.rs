use actix_multipart::Field;
use futures::StreamExt;
use std::collections::HashMap;

pub async fn extract_file_info(field: &mut Field) -> String {
    let filename = field.content_disposition()
        .get_filename()
        .unwrap_or("unknown")
        .to_owned();
    
    let mut size = 0;
    while let Some(chunk) = field.next().await {
        if let Ok(data) = chunk {
            size += data.len();
        }
    }
    format!("{}({} bytes)", filename, size)
}

pub async fn extract_form_field(field: &mut Field) -> (String, String) {
    let name = field.content_disposition()
        .get_name()
        .unwrap_or("unknown")
        .to_owned();
    
    let mut value = String::new();
    while let Some(chunk) = field.next().await {
        if let Ok(data) = chunk {
            if let Ok(s) = String::from_utf8(data.to_vec()) {
                value.push_str(&s);
            }
        }
    }
    (name, value)
}

pub fn create_response_message(form_data: &HashMap<String, String>, files_info: &Vec<String>) -> String {
    format!("Form processed with {} fields and {} files", 
        form_data.len(), 
        files_info.len()
    )
}
