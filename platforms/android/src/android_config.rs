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
        // Configuration des variables d'environnement pour Android
        std::env::set_var("SERVER_HOST", "127.0.0.1");
        std::env::set_var("SERVER_PORT", "8088");
        std::env::set_var("SSL_ENABLED", "false");
        std::env::set_var("ENVIRONMENT", "android");
        std::env::set_var("CORS_PERMISSIVE", "true");
        std::env::set_var("COMPRESSION_ENABLED", "true");
        std::env::set_var("SECURITY_HEADERS", "false"); // Relaxé pour Android
        std::env::set_var("FILE_CACHING", "true");
        std::env::set_var("REQUEST_LOGGING", "true");
        
        // Configuration de la base de données SQLite pour Android
        let db_path = self.database_path.join("rust_webassembly.sqlite");
        if let Some(db_str) = db_path.to_str() {
            let database_url = format!("sqlite:{}", db_str);
            std::env::set_var("DATABASE_URL", &database_url);
            println!("🗄️ Android DATABASE_URL set to: {}", database_url);
        }
        
        // Chemins spécifiques Android
        if let Some(assets_str) = self.assets_path.to_str() {
            std::env::set_var("ANDROID_ASSETS_PATH", assets_str);
        }
        if let Some(db_str) = self.database_path.to_str() {
            std::env::set_var("ANDROID_DATABASE_PATH", db_str);
        }
    }
    
    pub fn create_directories(&self) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(&self.assets_path)?;
        std::fs::create_dir_all(&self.database_path)?;
        std::fs::create_dir_all(&self.logs_path)?;
        Ok(())
    }
}
