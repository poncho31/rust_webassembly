use serde::{Deserialize, Serialize};
use std::env;

/// Configuration centralisée de l'application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub upload: UploadConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub port: u16,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadConfig {
    pub max_file_size: u64,
    pub allowed_extensions: Vec<String>,
    pub upload_dir: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(AppConfig {
            database: DatabaseConfig {
                host: env::var("PG_HOST")?,
                user: env::var("PG_USER")?,
                password: env::var("PG_PASSWORD")?,
                database: env::var("PG_DATABASE")?,
                port: env::var("PG_PORT")?.parse().unwrap_or(5432),
                max_connections: env::var("DB_MAX_CONNECTIONS")?.parse().unwrap_or(10),
                connection_timeout: env::var("DB_TIMEOUT")?.parse().unwrap_or(30),
            },
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("SERVER_PORT")?.parse().unwrap_or(8089),
                workers: env::var("SERVER_WORKERS")?.parse().unwrap_or(1),
                cors_origins: env::var("CORS_ORIGINS")
                    .unwrap_or_else(|_| "*".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
            upload: UploadConfig {
                max_file_size: env::var("MAX_FILE_SIZE")?.parse().unwrap_or(10 * 1024 * 1024), // 10MB
                allowed_extensions: env::var("ALLOWED_EXTENSIONS")
                    .unwrap_or_else(|_| "jpg,jpeg,png,pdf,txt".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                upload_dir: env::var("UPLOAD_DIR").unwrap_or_else(|_| "storage/files".to_string()),
            },
        })
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.user,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database
        )
    }
}
