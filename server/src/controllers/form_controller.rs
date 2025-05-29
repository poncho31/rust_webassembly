use actix_web::{HttpResponse, Error};
use actix_multipart::Multipart;
use futures::StreamExt;
use core::HttpSendResponse;
use std::collections::HashMap;
use serde_json::json;
use server::request_received::{extract_file_info, extract_form_field, create_response_message};

pub async fn post(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut form_data = HashMap::new();
    let mut files_info = Vec::new();

    while let Some(field) = payload.next().await {
        let mut field = field?;
        
        let field_name = field.content_disposition() .get_name().map(|s| s.to_owned());
        let filename = field.content_disposition() .get_filename().map(|s| s.to_owned());
        
        match (field_name, filename) {
            (Some(_), Some(_)) => {
                files_info.push(extract_file_info(&mut field).await);
            },
            (Some(_), None) => {
                let (name, value) = extract_form_field(&mut field).await;
                form_data.insert(name, value);
            },
            _ => continue,
        }
    }

    let data = json!({
        "form_fields": form_data,
        "files": files_info
    });

    let message = create_response_message(&form_data, &files_info);

    Ok(HttpResponse::Ok().json(HttpSendResponse {
        status: 200,
        message: Some(message),
        data: Some(data),
    }))
}
