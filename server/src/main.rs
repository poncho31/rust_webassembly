use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use std::env;
use actix_cors::Cors;

mod routes;
use crate::routes::ping_route;
use crate::routes::form_route;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8088".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a number");

    let static_path = std::env::current_dir()?.join("client").join("static");
    let pkg_path = std::env::current_dir()?.join("client").join("static").join("pkg");
    
    println!("Static files path: {:?}", static_path);
    println!("WASM pkg path: {:?}", pkg_path);
    
    assert!(static_path.exists(), "Static directory not found!");
    assert!(pkg_path.exists(), "WASM pkg directory not found!");
    assert!(static_path.join("index.html").exists(), "index.html not found!");

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
                    .route("/ping", web::get().to(ping_route::get))
                    .route("/form", web::post().to(form_route::post))
            )

            /*
             *  ███████ ████████  █████  ████████ ██  ██████  ███████
             *  ██         ██    ██   ██    ██    ██ ██       ██
             *  ███████    ██    ███████    ██    ██ ██       ███████
             *       ██    ██    ██   ██    ██    ██ ██            ██
             *  ███████    ██    ██   ██    ██    ██  ██████  ███████
             */
            .service(Files::new("/pkg", &pkg_path).show_files_listing())
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