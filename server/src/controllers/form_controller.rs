// Import necessary dependencies
use actix_web::{web, HttpResponse, Error};
use actix_multipart::Multipart;
use futures::StreamExt;
use core::{HttpSendResponse, DatabaseRepository};
use sqlx::PgPool;
use std::collections::HashMap;
use server::extract_form::{extract_form_field, save_uploaded_file};
use server::models::form_response::FormResponse;
use serde_json::to_value;
use tokio::time::{sleep, Duration};

/// Handles POST requests with multipart form data
/// Processes both file uploads and form fields
pub async fn post(
    mut payload: Multipart,
    db_pool: web::Data<PgPool>
) -> Result<HttpResponse, Error> {
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
            (Some(_name), Some(filename)) => {
                match save_uploaded_file(&mut field, &filename).await {
                    Ok(save_info) => {
                        files_info.push(format!("{} ({})", filename, save_info));
                    },
                    Err(e) => {
                        files_info.push(format!("Error saving {}: {}", filename, e));
                    }
                }
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

    // Créer le repository pour la base de données
    let repository = DatabaseRepository::new(db_pool.get_ref().clone());
    
    // Tentative de sauvegarde en base de données
    let database_result = match repository.upsert_user(&form_data, &files_info).await {
        Ok(form_data_saved) => {
            println!("User saved/updated successfully: {:?}", form_data_saved);
            Some(format!("User {} successfully saved to database", 
                form_data_saved.login.as_deref().unwrap_or("unknown")))
        },
        Err(e) => {
            println!("Database error: {}", e);
            Some(format!("Database error: {}", e))
        }
    };

    // Prepare response data combining form fields and file information
    let form_response = FormResponse {
        form_fields: form_data,
        files: files_info,
    };

    // Create message first, before consuming form_response
    let mut message = create_response_message(&form_response.form_fields, &form_response.files);
    
    // Ajouter le résultat de la base de données au message
    if let Some(db_message) = database_result {
        message.push_str(&format!("<br><strong>Database:</strong> {}", db_message));
    }

    // Then convert form_response to serde_json::Value
    let data = to_value(form_response).unwrap();

    // Add 500ms delay
    // sleep(Duration::from_millis(500)).await;
    
    // Return successful response with status, message and processed data
    Ok(HttpResponse::Ok().json(HttpSendResponse {
        status: 200,
        message: Some(message),
        data: Some(data),
    }))
}


pub fn create_response_message(form_data: &HashMap<String, String>, files_info: &Vec<String>) -> String {
    let fields_html = if form_data.is_empty() {
        "<tr><td>Fields</td><td>No fields</td></tr>".to_string()
    } else {
        form_data.iter()
            .map(|(key, value)| format!("<tr><td>{}</td><td>{}</td></tr>", key, value))
            .collect::<Vec<String>>()
            .join("\n")
    };

    let files_html = if files_info.is_empty() {
        "<tr><td>Files</td><td>No files</td></tr>".to_string()
    } else {
        format!("<tr><td>Files</td><td>{}</td></tr>",
            files_info.iter()
                .map(|file| file.to_string())
                .collect::<Vec<String>>()
                .join("<br>")
        )
    };

    format!(
        "<table class='response-table'><thead><tr><th>Field</th><th>Value</th></tr></thead><tbody>{}{}</tbody></table>",
        fields_html,
        files_html
    )
}
