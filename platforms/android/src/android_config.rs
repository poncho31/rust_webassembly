// Configuration Android pour le serveur embedded
use std::path::PathBuf;

pub struct AndroidServerConfig {
    pub assets_path: PathBuf,
    pub database_path: PathBuf,
    pub logs_path: PathBuf,
}

impl AndroidServerConfig {
    pub fn new() -> Self {
        // Chemins par défaut pour Android
        let base_path = PathBuf::from("/data/data/com.main/files");
        
        Self {
            assets_path: base_path.join("assets"),
            database_path: base_path.join("database"),
            logs_path: base_path.join("logs"),
        }
    }
    
    pub fn setup_android_environment(&self) {
        use log::info;
        
        info!("🔧 [CONFIG] Starting Android environment setup...");
        
        // FORCER l'écrasement de toutes les variables d'environnement pour Android
        // Bind to localhost pour Android (plus sécurisé que 0.0.0.0)
        std::env::set_var("SERVER_HOST", "127.0.0.1");
        std::env::set_var("SERVER_PORT", "8088");
        std::env::set_var("SSL_ENABLED", "false");
        std::env::set_var("ENVIRONMENT", "android");
        
        // Configuration CORS très permissive pour Android WebView
        std::env::set_var("CORS_PERMISSIVE", "true");
        std::env::set_var("CORS_ALLOW_ORIGIN", "*");
        std::env::set_var("CORS_ALLOW_METHODS", "GET,POST,PUT,DELETE,OPTIONS,HEAD");
        std::env::set_var("CORS_ALLOW_HEADERS", "*");
        std::env::set_var("CORS_ALLOW_CREDENTIALS", "true");
        std::env::set_var("CORS_MAX_AGE", "86400");
        
        // Configuration serveur Android
        std::env::set_var("COMPRESSION_ENABLED", "true");
        std::env::set_var("SECURITY_HEADERS", "false"); // Relaxé pour Android
        std::env::set_var("FILE_CACHING", "true");
        std::env::set_var("REQUEST_LOGGING", "true");
        
        // Configuration spéciale pour WebView Android
        std::env::set_var("ANDROID_WEBVIEW_MODE", "true");
        std::env::set_var("SERVE_STATIC_FILES", "true");
        std::env::set_var("ALLOW_FILE_PROTOCOL", "true");
        
        // FORCER la base de données SQLite en mémoire - ÉCRASER toute config .env
        let database_url = "sqlite://:memory:";
        std::env::set_var("DATABASE_URL", database_url);
        info!("🗄️ [CONFIG] Android DATABASE_URL FORCED to: {} (in-memory)", database_url);
        
        // Vérifier que la variable est bien définie
        let check_db_url = std::env::var("DATABASE_URL").unwrap_or("ERROR_NOT_SET".to_string());
        info!("🔍 [CONFIG] DATABASE_URL verification: {}", check_db_url);
        
        // Chemins spécifiques Android
        if let Some(assets_str) = self.assets_path.to_str() {
            std::env::set_var("ANDROID_ASSETS_PATH", assets_str);
            info!("🗂️ [CONFIG] ANDROID_ASSETS_PATH set to: {}", assets_str);
        }
        if let Some(db_str) = self.database_path.to_str() {
            std::env::set_var("ANDROID_DATABASE_PATH", db_str);
            info!("🗂️ [CONFIG] ANDROID_DATABASE_PATH set to: {}", db_str);
        }
        
        info!("✅ [CONFIG] Android environment setup completed");
    }
    
    pub fn create_directories(&self) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(&self.assets_path)?;
        std::fs::create_dir_all(&self.database_path)?;
        std::fs::create_dir_all(&self.logs_path)?;
        Ok(())
    }
    
    pub fn get_server_url(&self) -> String {
        let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8088".to_string());
        format!("http://{}:{}", host, port)
    }
    
    pub fn get_webview_url(&self) -> String {
        // Retourner l'URL du serveur au lieu du protocole file://
        format!("{}/", self.get_server_url())
    }
}
