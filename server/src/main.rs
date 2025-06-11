use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, middleware};
use actix_web::http::header;
use dotenv::dotenv;
use std::env;
use actix_cors::Cors;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sqlx::PgPool;

mod controllers;
mod ssl_config;

use crate::controllers::ping_controller;
use crate::controllers::index_controller;
use crate::controllers::weather_controller;
use crate::ssl_config::SslConfig;
use core::{init_db, create_database};




#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
      // Vérifier les arguments de commande
    let args: Vec<String> = env::args().collect();
    
    // Si il y a des arguments (cargo run -- COMMAND)
    if args.len() > 1 {
        let command = &args[1];
        return handle_command(command).await;
    }
    
    // Si pas d'arguments, lancer le serveur web normal
    println!("🚀 Démarrage du serveur web complet");
    start_full_web_server().await
}

/// Gère les commandes spécifiques
async fn handle_command(command: &str) -> std::io::Result<()> {
    match command {
        "ssl-info" => {
            println!("🔒 === INFORMATIONS SSL ===");
            SslConfig::print_ssl_info();
            Ok(())
        },
        "ssl-regen" => {
            println!("🔄 Régénération du certificat SSL...");
            match SslConfig::regenerate_certificate() {
                Ok(_) => println!("✅ Certificat SSL régénéré avec succès"),
                Err(e) => eprintln!("❌ Erreur lors de la régénération SSL: {}", e),
            }
            Ok(())
        },
        "config" => {
            show_server_config();
            Ok(())
        },
        "status" => {
            show_server_status();
            Ok(())
        },
        "routes" => {
            show_available_routes();
            Ok(())
        },
        "help" => {
            print_help();
            Ok(())
        },
        _ => {
            eprintln!("❌ Commande inconnue: {}", command);
            print_help();
            Ok(())
        }
    }
}

/// Affiche l'aide des commandes disponibles
fn print_help() {
    println!("🛠️  === COMMANDES DISPONIBLES ===");
    println!("  cargo run                - Démarre le serveur web complet");
    println!("  cargo run -- ssl-info   - Affiche les informations SSL");
    println!("  cargo run -- ssl-regen  - Régénère le certificat SSL");
    println!("  cargo run -- config     - Affiche la configuration serveur");
    println!("  cargo run -- status     - Affiche le statut du système");
    println!("  cargo run -- routes     - Liste toutes les routes disponibles");
    println!("  cargo run -- help       - Affiche cette aide");
    println!();
    println!("📋 === EXEMPLES ===");
    println!("  cargo run -- ssl-regen  # Régénère les certificats SSL");
    println!("  cargo run -- config     # Voir la config avant de démarrer");
    println!("  cargo run               # Lance le serveur web sur port 8088");
}

/// Affiche la configuration du serveur
fn show_server_config() {
    let config = create_web_server_config();
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    
    println!("⚙️  === CONFIGURATION SERVEUR ===");
    println!("🌐 Host: {}", config.host);
    println!("🔌 Port: {}", config.port);
    println!("🏗️  Environnement: {}", environment.to_uppercase());
    println!("👷 Workers: {}", config.workers);
    println!("🔗 Max Connections: {}", config.max_connections);
    println!("⏱️  Keep Alive: {:?}", config.keep_alive);
    println!("⏰ Client Timeout: {:?}", config.client_timeout);
    println!("🔒 SSL Enabled: {}", config.ssl_enabled);
    println!("📦 Compression: {}", config.compression_enabled);
    println!("🛡️  Security Headers: {}", config.security_headers_enabled);
    println!("🌍 CORS Permissive: {}", config.cors_permissive);
    println!("💾 File Caching: {}", config.file_caching);
    println!("📝 Request Logging: {}", config.request_logging);
    println!("================================");
}

/// Affiche le statut du système
fn show_server_status() {
    println!("📊 === STATUT DU SYSTÈME ===");
    println!("🔧 Version: {}", env!("CARGO_PKG_VERSION"));
    println!("🏗️  Build Profile: {}", if cfg!(debug_assertions) { "Debug" } else { "Release" });
    println!("💻 OS: {}", env::consts::OS);
    println!("🏛️  Architecture: {}", env::consts::ARCH);
    
    // Vérifications des chemins
    match get_static_path() {
        Ok((static_path, pkg_path, favicon_path)) => {
            println!("✅ Chemins statiques: OK");
            println!("   📁 Static: {:?}", static_path);
            println!("   📦 WASM Pkg: {:?}", pkg_path);
            println!("   🎨 Favicon: {:?}", favicon_path);
        },
        Err(e) => {
            println!("❌ Chemins statiques: ERREUR - {}", e);
        }
    }
    
    // Vérification SSL
    println!("🔒 SSL: {}", if SslConfig::is_ssl_ready() { "✅ Prêt" } else { "❌ Non configuré" });
    
    // Variables d'environnement importantes
    println!("🌍 Variables d'environnement:");
    let env_vars = [
        "SERVER_HOST", "SERVER_PORT", "SERVER_WORKERS", 
        "MAX_CONNECTIONS", "SSL_ENABLED", "ENVIRONMENT"
    ];
    
    for var in &env_vars {
        match env::var(var) {
            Ok(value) => println!("   {}: {}", var, value),
            Err(_) => println!("   {}: (défaut)", var),
        }
    }
    println!("===========================");
}

/// Affiche toutes les routes disponibles
fn show_available_routes() {
    println!("🔗 === ROUTES DISPONIBLES ===");    println!("📡 API Endpoints:");
    println!("   • POST /api/form              - Soumission de formulaire");
    println!("   • GET  /api/form_data         - Récupération des données form_data");
    println!("   • GET  /api/ping              - Test de santé du serveur");
    println!("   • POST /api/ping              - Test de santé du serveur");
    println!("   • GET  /api/weather/temperature - Données météo");
    println!("   • GET  /health                - Point de santé système");
    println!();
    println!("📄 Pages statiques:");
    println!("   • GET  /                      - Page d'accueil (index.html)");
    println!("   • GET  /test                  - Page de test (test.html)");
    println!("   • GET  /pkg/*                 - Fichiers WebAssembly");
    println!("   • GET  /favicon.ico           - Icône du site");
    println!();
    println!("🔧 Utilitaires:");
    println!("   • GET  /*                     - Fichiers statiques");
    println!("   • *    (404)                  - Gestion des erreurs");
    println!("=============================");
}

/// Démarre le serveur web complet
async fn start_full_web_server() -> std::io::Result<()> {
    // Configuration centralisée du serveur web
    let config = create_web_server_config();
    let (static_path, pkg_path, favicon_path) = get_static_path().expect("Failed to initialize static paths");
    
    // Affichage des informations du serveur
    print_server_info(&config);    // Configuration SSL si activée
    if config.ssl_enabled {
        SslConfig::print_ssl_info();
    }

    // Initialisation de la base de données
    println!("🗄️ Initializing database connection...");
    let db_pool = match init_database().await {
        Ok(pool) => {
            println!("✅ Database connection established successfully");
            pool
        },
        Err(e) => {
            println!("⚠️ Warning: Could not connect to database: {}", e);
            println!("⚠️ Server will continue without database functionality");
            // Créer un pool vide ou utiliser une option
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
    let ssl_enabled = config.ssl_enabled;    let http_server_instance = HttpServer::new(move || {
        let cors = configure_cors(&config);
        
        // Application de base avec tous les middlewares essentiels
        let app = App::new()
            .app_data(web::Data::new(db_pool.clone()))
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
                .route("/form", web::post().to(index_controller::post))
                .route("/form_data", web::get().to(index_controller::get_form_data))
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
        http_server_instance.bind((host, port))?    };
    
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
    println!("");    println!("🔧 API Endpoints:");
    println!("   • GET/POST /api/ping - Server health check");
    println!("   • POST /api/form - Form submission");
    println!("   • GET /api/form_data - Retrieve form_data table");
    println!("   • GET /api/weather/temperature - Weather data");
    println!("   • GET /health - Health check endpoint");
    println!("=====================================");
}


fn get_static_path() -> std::io::Result<(std::path::PathBuf, std::path::PathBuf, std::path::PathBuf)> {
    // Déterminer le répertoire racine du projet
    let current_dir = std::env::current_dir()?;
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

/// Initialise la connexion à la base de données
async fn init_database() -> Result<PgPool, Box<dyn std::error::Error>> {
    println!("Creating database if it doesn't exist...");
    if let Err(e) = create_database().await {
        println!("Warning: Could not ensure database exists: {}", e);
    }

    println!("Initializing database connection pool...");
    match init_db().await {
        Ok(pool) => {
            println!("Database pool created successfully");
            Ok(pool)
        },        Err(e) => {
            println!("Failed to initialize database: {}", e);
            Err(format!("Database initialization failed: {}", e).into())
        }
    }
}