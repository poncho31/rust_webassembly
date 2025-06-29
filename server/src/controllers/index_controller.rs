// Import necessary dependencies
use actix_web::{web, HttpResponse, Error};
use actix_multipart::Multipart;
use futures::StreamExt;
use core::{HttpSendResponse, UserRepository, Table, _database::DatabaseQuery};
use std::collections::HashMap;
use server_lib::extract_form::{extract_form_field, save_uploaded_file};
use server_lib::models::form_response::FormResponse;
use serde_json::{to_value, value, Value};

/// Handles POST requests with multipart form data
/// Processes both file uploads and form fields
pub async fn post(
    mut payload: Multipart,
    db_pool: web::Data<DatabaseQuery>
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
    }    // Créer le repository pour la base de données
    let repository = UserRepository::new(db_pool.get_ref().clone());
    
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
    
    // Prepare response data combining form fields, files and database result
    let mut response_data = serde_json::Map::new();

    if let Some(ref db_msg) = database_result {
        response_data.insert("Message".to_string(), Value::String(db_msg.clone()));
    }

    if !files_info.is_empty() {
        response_data.insert("Fichiers".to_string(),       to_value(files_info.clone()).unwrap());
    }

    response_data.insert("Données".to_string(), to_value(form_data.clone()).unwrap());
    
    let data = Value::Object(response_data);
    
    // Generate HTML table using client's table generator
    let message: String = Table::create(&data, "response-table").to_html();

    // Also create the original FormResponse for backward compatibility
    let form_response = FormResponse {
        form_fields: form_data.clone(),
        files: files_info.clone(),
    };
    let form_response_data = to_value(form_response).unwrap();

    // Return successful response with status, message and processed data
    Ok(HttpResponse::Ok().json(HttpSendResponse {
        status: 200,
        message: Some(message),
        data: Some(form_response_data),
    }))
}

/// Récupère toutes les données de la table form_data
pub async fn get_form_data(
    db_pool: web::Data<DatabaseQuery>
) -> Result<HttpResponse, Error> {
    // Créer le repository pour la base de données
    let user_repo: UserRepository = UserRepository::new(db_pool.get_ref().clone());
      // Récupérer toutes les données
    match user_repo.get_all().await {
        Ok(form_data_list) => {
            // Convertir en JSON pour Table::create() - wrap dans un objet avec une clé
            let wrapped_data = serde_json::json!({"form_data": form_data_list});
            
            // Générer le HTML de la table avec les outils existants
            let table_html = Table::create(&wrapped_data, "form-data-table").to_html();
            
            // Retourner le HTML directement
            Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body(table_html))
        },
        Err(e) => {
            println!("Database error: {}", e);
            let error_table = Table::create(&serde_json::json!({
                "error": "Failed to fetch form data",
                "details": e.to_string()
            }), "error-table").to_html();
            
            Ok(HttpResponse::InternalServerError()
                .content_type("text/html")
                .body(error_table))
        }
    }
}
