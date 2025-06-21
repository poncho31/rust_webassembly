use actix_web::{web, App, HttpResponse, HttpServer, middleware, Result};
use actix_files::Files;
use actix_cors::Cors;
use serde_json::json;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8088".to_string())
        .parse::<u16>()
        .unwrap_or(8088);
    
    println!("🚀 Starting Android HTTP Server on http://{}:{}", host, port);
    println!("📁 Serving static files from current directory");
    
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .wrap(middleware::Logger::default())
            // API Routes (simples pour Android)
            .service(web::scope("/api")
                .route("/ping", web::get().to(ping_handler))
                .route("/ping", web::post().to(ping_handler))
                .route("/form", web::post().to(form_handler))
                .route("/form_data", web::get().to(form_data_handler))
                .route("/weather/temperature", web::get().to(weather_handler))
            )
            // Servir les fichiers statiques depuis le répertoire static
            .service(Files::new("/", "./static").index_file("index.html"))
            .default_service(web::get().to(|| async {
                HttpResponse::NotFound().body("404 - Page not found")
            }))
    })
    .bind((host, port))?
    .run()
    .await
}

async fn ping_handler() -> Result<HttpResponse> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "message": "Android server is running",
        "timestamp": timestamp
    })))
}

async fn form_handler(form_data: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    println!("📋 Form data received: {:?}", form_data);
    
    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Form data received successfully",
        "data": *form_data
    })))
}

async fn form_data_handler() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "users": [
            {"id": 1, "name": "John Doe", "email": "john@example.com"},
            {"id": 2, "name": "Jane Smith", "email": "jane@example.com"}
        ]
    })))
}

async fn weather_handler() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "temperature": 22.5,
        "humidity": 65,
        "condition": "Sunny"
    })))
}
