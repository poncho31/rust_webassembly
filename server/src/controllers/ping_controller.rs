use actix_web::HttpResponse;
use core::{User, HttpSendResponse};

pub async fn get() -> HttpResponse {
    println!("Ping request received!");

    let message    = format!("Ping request received !");
    
    HttpResponse::Ok().json(HttpSendResponse {
        status: 200,
        message: Some(message),
        data: None,
    })
}
