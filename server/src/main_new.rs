use dotenv::dotenv;
use std::env;

mod controllers;
mod ssl_config;
mod server_lib;

use server_lib::{start_full_web_server, create_web_server_config};
use ssl_config::SslConfig;

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
    println!("🔗 === ROUTES DISPONIBLES ===");    
    println!("📡 API Endpoints:");
    println!("   • POST /api/form              - Soumission de formulaire");
    println!("   • GET  /api/form_data         - Récupération des données form_data");
    println!("   • GET  /api/ping              - Test de santé du serveur");
    println!("   • POST /api/ping              - Test de santé du serveur");
    println!("   • GET  /api/weather/temperature - Données météo");
    println!();
    println!("📄 Pages statiques:");
    println!("   • GET  /                      - Page d'accueil (index.html)");
    println!("   • GET  /pkg/*                 - Fichiers WebAssembly");
    println!("   • GET  /favicon.ico           - Icône du site");
    println!();
    println!("🔧 Utilitaires:");
    println!("   • GET  /*                     - Fichiers statiques");
    println!("   • *    (404)                  - Gestion des erreurs");
    println!("=============================");
}
