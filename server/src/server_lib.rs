// Ce module contient toute la logique du serveur web
// Il peut être utilisé à la fois par main.rs et par d'autres composants comme Android

use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, middleware};
use actix_web::http::header;
use dotenv::dotenv;
use std::env;
use actix_cors::Cors;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use core::_database::DatabaseQuery;
use serde_json;

// Import des contrôleurs - ils doivent être accessibles depuis ce module
use crate::controllers::ping_controller;
use crate::controllers::index_controller;
use crate::controllers::weather_controller;
use crate::ssl_config::SslConfig;

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
            host: "localhost".to_string(),
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

pub fn create_web_server_config() -> WebServerConfig {
    WebServerConfig {
        host: env::var("SERVER_HOST")
            .unwrap_or("localhost".to_string()),

        port: env::var("SERVER_PORT")
            .unwrap_or("8088".to_string())
            .parse::<u16>()
            .expect("SERVER_PORT must be a valid number"),

        workers: env::var("SERVER_WORKERS")
            .unwrap_or("1".to_string())
            .parse::<usize>()
            .unwrap_or(1),

        max_connections: env::var("MAX_CONNECTIONS")
            .unwrap_or( "1000".to_string())
            .parse::<usize>()
            .unwrap_or(1000),

        ssl_enabled: env::var("SSL_ENABLED")
            .unwrap_or("true".to_string())
            .parse::<bool>()
            .unwrap_or(true),

        compression_enabled: env::var("COMPRESSION_ENABLED")
            .unwrap_or( "true".to_string())
            .parse::<bool>()
            .unwrap_or(true),

        security_headers_enabled: env::var("SECURITY_HEADERS")
            .unwrap_or( "true".to_string())
            .parse::<bool>()
            .unwrap_or(true),

        cors_permissive: env::var("CORS_PERMISSIVE")
            .unwrap_or( "false".to_string())
            .parse::<bool>()
            .unwrap_or(false) ,

        file_caching: env::var("FILE_CACHING")
            .unwrap_or( "true".to_string())
            .parse::<bool>()
            .unwrap_or(true),

        request_logging: env::var("REQUEST_LOGGING")
            .unwrap_or( "true".to_string())
            .parse::<bool>()
            .unwrap_or(true),

        ..Default::default()
    }
}

/// Démarre le serveur web complet
pub async fn start_full_web_server() -> std::io::Result<()> {
    // Configuration centralisée du serveur web
    let config = create_web_server_config();
    let (static_path, pkg_path, favicon_path) = get_static_path().expect("Failed to initialize static paths");
    
    // Affichage des informations du serveur
    print_server_info(&config);
    
    // Configuration SSL si activée
    if config.ssl_enabled {
        SslConfig::print_ssl_info();
    }

    // Initialisation de la base de données - OBLIGATOIRE pour le démarrage
    println!("🗄️ Initializing database connection...");
    let db_pool = match core::_database::init_db().await {
        Ok(pool) => {
            println!("✅ Database connection established successfully");
            pool
        },
        Err(e) => {
            println!("❌ FATAL ERROR: Could not connect to database: {}", e);
            println!("❌ Database connection is REQUIRED for server startup");
            println!("❌ Please check your DATABASE_URL and ensure the database is accessible");
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("Database connection failed: {}", e)
            ));
        }
    };

    // Copier les valeurs nécessaires avant le move
    let host = config.host.clone();
    let port = config.port;
    let workers = config.workers;
    let max_connections = config.max_connections;
    let keep_alive = config.keep_alive;
    let client_timeout = config.client_timeout;
    let ssl_enabled = config.ssl_enabled;
    
    let http_server_instance = HttpServer::new(move || {
        let cors = configure_cors(&config);
        
        // Application de base avec tous les middlewares essentiels
        let mut default_headers = middleware::DefaultHeaders::new()
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
            .add((
                "Content-Security-Policy",
                "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' data:; font-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'self'; connect-src 'self'; media-src 'self'; child-src 'none'; form-action 'self'; manifest-src 'self'; worker-src 'self';"
            ));
        if config.ssl_enabled {
            default_headers = default_headers.add((
                "Strict-Transport-Security",
                "max-age=63072000; includeSubDomains; preload"
            ));
        }
        let app = App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .wrap(cors)
            .wrap(middleware::Compress::default())
            .wrap(default_headers);
        
        app
            // Routes de l'API
            .service(web::scope("/api")
                .route("/form", web::post().to(index_controller::post))
                .route("/form_data", web::get().to(index_controller::get_form_data))
                .route("/ping", web::post().to(ping_controller::get))
                .route("/ping", web::get().to(ping_controller::get))
                .route("/weather/temperature", web::get().to(weather_controller::get_temperature))
            )
            
            // Fichier index.html 
            .service({
                let mut files = Files::new("/", &static_path)
                    .index_file(env::var("HTML_INDEX").unwrap_or_else(|_| "index.html".to_string()));
                
                if config.file_caching {
                    files = files.use_etag(true).use_last_modified(true);
                }
                files
            })
            // fichier pkg
            .service({
                let mut files = Files::new("/pkg", &pkg_path).show_files_listing();
                if config.file_caching {
                    files = files.use_etag(true).use_last_modified(true);
                }
                files
            })
            // Fichier favicon.ico
            .service(Files::new("/favicon.ico", &favicon_path))
            .default_service(web::get().to(|| async {
                        actix_files::NamedFile::open_async("client/static/404.html").await
                    }))
    })
    .workers(workers)
    .max_connections(max_connections)
    .keep_alive(keep_alive)
    .client_request_timeout(client_timeout);

    // Configuration SSL conditionnelle
    let http_server = if ssl_enabled {
        match SslConfig::create_ssl_acceptor() {
            Ok(ssl_config) => {
                println!("🔒 Serveur HTTPS démarré sur https://{}:{}", host, port);
                http_server_instance.bind_rustls_0_22((host, port), ssl_config)?
            },
            Err(e) => {
                eprintln!("❌ Erreur SSL: {}. Démarrage en HTTP...", e);
                println!("🔓 Serveur HTTP démarré sur http://{}:{}", host, port);
                http_server_instance.bind((host, port))?
            }
        }
    } else {
        println!("🔓 Serveur HTTP démarré sur http://{}:{}", host, port);
        http_server_instance.bind((host, port))?
    };
    
    http_server.run().await
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

fn print_server_info(config: &WebServerConfig) {
    let protocol = if config.ssl_enabled { "https" } else { "http" };
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    
    println!("🌐 ===== WEB SERVER CONFIGURATION =====");
    println!("🚀 Server starting on {}://{}:{}", protocol, config.host, config.port);
    println!("🏗️ Environment: {}", environment.to_uppercase());
    println!("👷 Workers: {}", config.workers);
    println!("🔗 Max Connections: {}", config.max_connections);
    println!("⏱️ Keep Alive: {:?}", config.keep_alive);
    println!("🔒 SSL Enabled: {}", config.ssl_enabled);
    println!("📦 Compression: {}", config.compression_enabled);
    println!("🛡️ Security Headers: {}", config.security_headers_enabled);
    println!("🌍 CORS Permissive: {}", config.cors_permissive);
    println!("💾 File Caching: {}", config.file_caching);
    println!("📝 Request Logging: {}", config.request_logging);
    println!("");    
    println!("🔧 API Endpoints:");
    println!("   • GET/POST /api/ping           - Server health check");
    println!("   • POST /api/form               - Form submission");
    println!("   • GET /api/form_data           - Retrieve form_data table");
    println!("   • GET /api/weather/temperature - Weather data");
    println!("=====================================");
}

fn get_static_path() -> std::io::Result<(std::path::PathBuf, std::path::PathBuf, std::path::PathBuf)> {
    // Déterminer le répertoire racine du projet
    let current_dir = std::env::current_dir()?;
    
    // Vérifier si on est sur Android en cherchant les variables d'environnement Android
    let is_android = std::env::var("ENVIRONMENT").unwrap_or_default() == "android";
    
    if is_android {
        println!("🤖 Android environment detected, using Android asset paths...");
        
        // Chemins spécifiques pour Android - utiliser les assets copiés pendant le build
        let potential_android_paths = vec![
            // Chemins natifs Android à l'exécution
            std::path::PathBuf::from("/android_asset/static"),
            // Chemins de développement (pendant la compilation/test)
            std::path::PathBuf::from("./app/src/main/assets/static"),
            std::path::PathBuf::from("../app/src/main/assets/static"),  
            std::path::PathBuf::from("../../app/src/main/assets/static"),
            // Chemins relatifs possibles
            std::path::PathBuf::from("./assets/static"),
            std::path::PathBuf::from("../assets/static"),
            // Fallback vers les sources
            std::path::PathBuf::from("../../../client/static"),
            std::path::PathBuf::from("../../client/static"),
        ];
        
        println!("🔍 Searching for Android static files...");
        println!("🔍 Current working directory: {:?}", current_dir);
        
        for path in potential_android_paths {
            println!("🔍 Checking path: {:?}", path);
            if path.exists() {
                println!("✅ Path exists: {:?}", path);
                if path.join("index.html").exists() {
                    println!("✅ index.html found in: {:?}", path);
                    let pkg_path = path.join("pkg");
                    let favicon_path = path.join("images").join("icons").join("favicon.ico");
                    
                    println!("✅ Using Android static path: {:?}", path);
                    println!("✅ Using Android pkg path: {:?}", pkg_path);
                    println!("✅ Using Android favicon path: {:?}", favicon_path);
                    
                    return Ok((path, pkg_path, favicon_path));
                } else {
                    println!("❌ index.html not found in: {:?}", path);
                    // Lister le contenu pour debug
                    if let Ok(entries) = std::fs::read_dir(&path) {
                        let files: Vec<String> = entries
                            .filter_map(|entry| entry.ok())
                            .map(|entry| entry.file_name().to_string_lossy().to_string())
                            .collect();
                        println!("   📁 Directory contents: {:?}", files);
                    }
                }
            } else {
                println!("❌ Path does not exist: {:?}", path);
            }
        }
        
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound, 
            "Android static files not found! Make sure the build script copied client/static to app/src/main/assets/static"
        ));
    }
    
    // Code normal pour desktop
    let project_root = if current_dir.ends_with("server") {
        // Si on est dans le dossier server, remonter au parent
        current_dir.parent().unwrap().to_path_buf()
    } else {
        // Sinon, on est déjà à la racine du projet
        current_dir
    };
    
    let static_path  = project_root.join("client").join("static");
    let pkg_path     = static_path.join("pkg");
    let favicon_path = static_path.join("images").join("icons").join("favicon.ico");
    
    println!("Current dir: {:?}", std::env::current_dir()?);
    println!("Project root: {:?}", project_root);
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


