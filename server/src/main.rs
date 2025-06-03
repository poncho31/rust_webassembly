use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, middleware};
use actix_web::http::header;
use dotenv::dotenv;
use std::env;
use actix_cors::Cors;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod controllers;
use crate::controllers::ping_controller;
use crate::controllers::form_controller;
use crate::controllers::weather_controller;




#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    
    // Configuration centralisée du serveur web
    let config = create_web_server_config();
    let (static_path, pkg_path, favicon_path) = get_static_path().expect("Failed to initialize static paths");
      // Affichage des informations du serveur
    print_server_info(&config);

    // Copier les valeurs nécessaires avant le move
    let host = config.host.clone();
    let port = config.port;
    let workers = config.workers;
    let max_connections = config.max_connections;
    let keep_alive = config.keep_alive;
    let client_timeout = config.client_timeout;    let http_server_instance = HttpServer::new(move || {
        let cors = configure_cors(&config);
        
        // Application de base avec tous les middlewares essentiels
        let app = App::new()
            .wrap(cors)
            .wrap(middleware::Compress::default())
            .wrap(middleware::DefaultHeaders::new()
                .add(("X-Content-Type-Options", "nosniff"))
                .add((
                    "X-Frame-Options", 
                    if config.security_headers_enabled { "DENY" } else { "SAMEORIGIN" }
                ))
                .add((
                    "X-XSS-Protection", 
                    if config.security_headers_enabled { "1; mode=block" } else { "1" }
                ))
                .add((
                    "Referrer-Policy", 
                    if config.security_headers_enabled { "strict-origin-when-cross-origin" } else { "same-origin" }
                ))
            );
        
        app
            /*
            *  ██████   ██████  ██  ██ ████████ ███████  ███████
            *  ██   ██ ██    ██ ██  ██    ██    ██       ██
            *  ██████  ██    ██ ██  ██    ██    █████    ███████
            *  ██   ██ ██    ██ ██  ██    ██    ██            ██
            *  ██   ██  ██████   ████     ██    ███████  ██████
            */
            .service(web::scope("/api")
                .route("/form", web::post().to(form_controller::post))
                .route("/ping", web::post().to(ping_controller::get))
                .route("/ping", web::get().to(ping_controller::get))
                .route("/weather/temperature", web::get().to(weather_controller::get_temperature))
            )
            
            // Health check endpoint
            .route("/health", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({
                    "status": "healthy",
                    "timestamp": get_current_timestamp(),
                    "version": env!("CARGO_PKG_VERSION")
                }))
            }))

            /*
             *  ███████ ████████  █████  ████████ ██  ██████  ███████
             *  ██         ██    ██   ██    ██    ██ ██       ██
             *  ███████    ██    ███████    ██    ██ ██       ███████
             *       ██    ██    ██   ██    ██    ██ ██            ██
             *  ███████    ██    ██   ██    ██    ██  ██████  ███████
             */
            .service({
                let mut files = Files::new("/test", &static_path).index_file("test.html".to_string());
                if config.file_caching {
                    files = files.use_etag(true).use_last_modified(true);
                }
                files
            })
            .service({
                let mut files = Files::new("/", &static_path)
                    .index_file(env::var("HTML_INDEX").unwrap_or_else(|_| "index.html".to_string()));
                if config.file_caching {
                    files = files.use_etag(true).use_last_modified(true);
                }
                files
            })
            .service({
                let mut files = Files::new("/pkg", &pkg_path).show_files_listing();
                if config.file_caching {
                    files = files.use_etag(true).use_last_modified(true);
                }
                files
            })
            .service(Files::new("/favicon.ico", &favicon_path))
            .default_service(web::route().to(|| async {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Resource not found",
                    "status": 404,
                    "timestamp": get_current_timestamp()
                }))
            }))    })
    .workers(workers)
    .max_connections(max_connections)    .keep_alive(keep_alive)
    .client_request_timeout(client_timeout);
    
    let http_server = http_server_instance.bind((host, port))?;
    
    http_server.run().await
}




#[derive(Clone)]
pub struct WebServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_connections: usize,
    pub keep_alive: Duration,
    pub client_timeout: Duration,
    pub ssl_enabled: bool,
    pub compression_enabled: bool,
    pub security_headers_enabled: bool,
    pub cors_permissive: bool,
    pub file_caching: bool,
    pub request_logging: bool,
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8088,
            workers: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
            max_connections: 25000,
            keep_alive: Duration::from_secs(75),
            client_timeout: Duration::from_secs(5000),
            ssl_enabled: false,
            compression_enabled: true,
            security_headers_enabled: true,
            cors_permissive: true,
            file_caching: true,
            request_logging: true,
        }
    }
}

fn create_web_server_config() -> WebServerConfig {
    let is_production = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()) == "production";
    let default_workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    
    WebServerConfig {
        host: env::var("SERVER_HOST").unwrap_or_else(|_| {
            if is_production { "0.0.0.0".to_string() } else { "127.0.0.1".to_string() }
        }),
        port: env::var("SERVER_PORT")
            .unwrap_or_else(|_| if is_production { "80".to_string() } else { "8088".to_string() })
            .parse::<u16>()
            .expect("SERVER_PORT must be a valid number"),
        workers: env::var("SERVER_WORKERS")
            .unwrap_or_else(|_| {
                if is_production { 
                    (default_workers * 2).to_string() 
                } else { 
                    "1".to_string() 
                }
            })
            .parse::<usize>()
            .unwrap_or(if is_production { default_workers * 2 } else { 1 }),
        max_connections: env::var("MAX_CONNECTIONS")
            .unwrap_or_else(|_| if is_production { "50000".to_string() } else { "1000".to_string() })
            .parse::<usize>()
            .unwrap_or(if is_production { 50000 } else { 1000 }),
        ssl_enabled: env::var("SSL_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false),
        compression_enabled: env::var("COMPRESSION_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true),
        security_headers_enabled: env::var("SECURITY_HEADERS")
            .unwrap_or_else(|_| if is_production { "true".to_string() } else { "false".to_string() })
            .parse::<bool>()
            .unwrap_or(is_production),
        cors_permissive: !is_production,
        file_caching: env::var("FILE_CACHING")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true),
        request_logging: env::var("REQUEST_LOGGING")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true),
        ..Default::default()
    }
}

fn configure_cors(config: &WebServerConfig) -> Cors {
    if config.cors_permissive {
        Cors::permissive()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .expose_headers(vec!["content-type", "content-length", "accept"])
            .max_age(3600)
    } else {
        Cors::default()
            .allowed_origin(&env::var("ALLOWED_ORIGIN").unwrap_or_else(|_| "https://yourdomain.com".to_string()))
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
            .max_age(3600)
    }
}

fn get_current_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

fn print_server_info(config: &WebServerConfig) {
    let protocol = if config.ssl_enabled { "https" } else { "http" };
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    
    println!("🌐 ===== WEB SERVER CONFIGURATION =====");
    println!("🚀 Server starting on {}://{}:{}", protocol, config.host, config.port);
    println!("🏗️  Environment: {}", environment.to_uppercase());
    println!("👷 Workers: {}", config.workers);
    println!("🔗 Max Connections: {}", config.max_connections);
    println!("⏱️  Keep Alive: {:?}", config.keep_alive);
    println!("🔒 SSL Enabled: {}", config.ssl_enabled);
    println!("📦 Compression: {}", config.compression_enabled);
    println!("🛡️  Security Headers: {}", config.security_headers_enabled);
    println!("🌍 CORS Permissive: {}", config.cors_permissive);
    println!("💾 File Caching: {}", config.file_caching);
    println!("📝 Request Logging: {}", config.request_logging);
    println!("");
    println!("🔧 API Endpoints:");
    println!("   • GET/POST /api/ping - Server health check");
    println!("   • POST /api/form - Form submission");
    println!("   • GET /api/weather/temperature - Weather data");
    println!("   • GET /health - Health check endpoint");
    println!("=====================================");
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