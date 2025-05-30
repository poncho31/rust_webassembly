use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use std::env;
use actix_cors::Cors;

mod controllers;
use crate::controllers::ping_controller;
use crate::controllers::form_controller;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    
    let (host, port) = get_server_config();
    let (static_path, pkg_path, favicon_path) = get_static_path().expect("Failed to initialize static paths");


    HttpServer::new(move || {
        let cors = Cors::permissive()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .expose_headers(vec!["content-type", "content-length", "accept"])
            .max_age(3600);

        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(cors)

            /*
            *  ██████   ██████  ██  ██ ████████ ███████  ███████
            *  ██   ██ ██    ██ ██  ██    ██    ██       ██
            *  ██████  ██    ██ ██  ██    ██    █████    ███████
            *  ██   ██ ██    ██ ██  ██    ██    ██            ██
            *  ██   ██  ██████   ████     ██    ███████  ███████
            */
            .service(
                web::scope("/api")
                    .route("/ping", web::get().to(ping_controller::get))
                    .route("/ping", web::post().to(ping_controller::get))
                    .route("/form", web::post().to(form_controller::post))
            )

            /*
             *  ███████ ████████  █████  ████████ ██  ██████  ███████
             *  ██         ██    ██   ██    ██    ██ ██       ██
             *  ███████    ██    ███████    ██    ██ ██       ███████
             *       ██    ██    ██   ██    ██    ██ ██            ██
             *  ███████    ██    ██   ██    ██    ██  ██████  ███████
             */
            .service(Files::new("/pkg", &pkg_path).show_files_listing())
            .service(Files::new("/favicon.ico", &favicon_path))
            .service(Files::new("/", &static_path).index_file(env::var("HTML_INDEX").unwrap_or_else(|_| "index.html".to_string())))
            .default_service(web::route().to(|| async {
                HttpResponse::NotFound().body("Resource not found")
            }))
    })
    .workers(1)
    .bind((host, port))?
    .run()
    .await
}

fn get_server_config() -> (String, u16) {
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8088".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a number");
    
    (host, port)
}

fn get_static_path() -> std::io::Result<(std::path::PathBuf, std::path::PathBuf, std::path::PathBuf)> {
    let static_path  = std::env::current_dir()?.join("client").join("static");
    let pkg_path     = static_path.join("pkg");
    let favicon_path = static_path.join("images").join("icons").join("favicon.ico");
    
    println!("Static files path: {:?}", static_path);
    println!("favicon_path files path: {:?}", favicon_path);
    println!("WASM pkg path: {:?}", pkg_path);
    
    if !static_path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Static directory not found!"));
    }
    if !pkg_path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "WASM pkg directory not found!"));
    }
    if !static_path.join("index.html").exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "index.html not found!"));
    }

    Ok((static_path, pkg_path, favicon_path))
}