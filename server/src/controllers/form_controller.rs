// Import necessary dependencies
use actix_web::{HttpResponse, Error};
use actix_multipart::Multipart;
use futures::StreamExt;
use core::HttpSendResponse;
use std::collections::HashMap;
use server::extract_form::{extract_file_info, extract_form_field};
use server::models::form_response::FormResponse;
use serde_json::to_value;

/// Handles POST requests with multipart form data
/// Processes both file uploads and form fields
pub async fn post(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // Store form fields and file information
    let mut form_data = HashMap::new();
    let mut files_info = Vec::new();

    // Process each field in the multipart form
    while let Some(field) = payload.next().await {
        let mut field = field?;
        
        // Extract field name and filename from content disposition
        let field_name = field.content_disposition().get_name().map(|s| s.to_owned());
        let filename   = field.content_disposition().get_filename().map(|s| s.to_owned());
        
        match (field_name, filename) {
            // Handle file upload fields (has both name and filename)
            (Some(_), Some(_)) => {
                files_info.push(extract_file_info(&mut field).await);
            },
            // Handle regular form fields (has name but no filename)
            (Some(_), None) => {
                let (name, value) = extract_form_field(&mut field).await;
                form_data.insert(name, value);
            },
            // Skip fields without a name
            _ => continue,
        }
    }

    // Prepare response data combining form fields and file information
    let form_response = FormResponse {
        form_fields: form_data,
        files: files_info,
    };

    // Create message first, before consuming form_response
    let message = create_response_message(&form_response.form_fields, &form_response.files);

    // Then convert form_response to serde_json::Value
    let data = to_value(form_response).unwrap();

    // Return successful response with status, message and processed data
    Ok(HttpResponse::Ok().json(HttpSendResponse {
        status: 200,
        message: Some(message),
        data: Some(data),
    }))
}


pub fn create_response_message(form_data: &HashMap<String, String>, files_info: &Vec<String>) -> String {
    let fields_list = if form_data.is_empty() {
        "no fields".to_string()
    } else {
        form_data.keys()
            .map(|k| k.as_str())
            .collect::<Vec<&str>>()
            .join(", ")
    };

    let files_list = if files_info.is_empty() {
        "no files".to_string()
    } else {
        files_info.join(", ")
    };

    format!("Form processed with fields: [{}] and files: [{}]", 
        fields_list,
        files_list
    )
}
