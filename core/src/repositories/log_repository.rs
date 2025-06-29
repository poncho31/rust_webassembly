use sqlx::Row;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use time::OffsetDateTime;
use uuid::Uuid;
use crate::repositories::_database::DatabaseQuery;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Log {
    pub id: Uuid,
    pub r#type: String,        
    pub level: i32,
    pub message: String,
    pub context: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, Copy)]
pub enum LogLevel {
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    pub fn as_i32(&self) -> i32 {
        self.clone() as i32
    }
    
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(LogLevel::Debug),
            2 => Some(LogLevel::Info),
            3 => Some(LogLevel::Warn),
            4 => Some(LogLevel::Error),
            5 => Some(LogLevel::Fatal),
            _ => None,
        }
    }
}

pub struct LogRepository {
    db: DatabaseQuery,
}

impl Log {
    pub fn new(
        log_type: &str,
        level: LogLevel,
        message: &str,
        context: Option<&str>,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Uuid::new_v4(),
            r#type: log_type.to_string(),
            level: level.as_i32(),
            message: message.to_string(),
            context: context.map(|s| s.to_string()),
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn info(log_type: &str, message: &str) -> Self {
        Self::new(log_type, LogLevel::Info, message, None)
    }
    
    pub fn error(log_type: &str, message: &str, context: Option<&str>) -> Self {
        Self::new(log_type, LogLevel::Error, message, context)
    }
    
    pub fn debug(log_type: &str, message: &str, context: Option<&str>) -> Self {
        Self::new(log_type, LogLevel::Debug, message, context)
    }
    
    pub fn warn(log_type: &str, message: &str, context: Option<&str>) -> Self {
        Self::new(log_type, LogLevel::Warn, message, context)
    }
}

impl LogRepository {
    pub fn new(db_query: DatabaseQuery) -> Self {
        Self { db: db_query }
    }

    /// Crée un nouveau log
    pub async fn create_log(&self, log: &Log) -> Result<Log> {
        let query = format!(
            "INSERT INTO logs (id, type, level, message, context, created_at, updated_at) VALUES ('{}', '{}', {}, '{}', '{}', '{}', '{}')",
            log.id,
            log.r#type,
            log.level,
            log.message,
            log.context.as_ref().unwrap_or(&"".to_string()),
            log.created_at,
            log.updated_at
        );
        self.db.run_query(&query).await?;
        Ok(log.clone())
    }

    /// Méthode utilitaire pour log d'information
    pub async fn log_info(&self, log_type: &str, message: &str) -> Result<Log> {
        let log = Log::info(log_type, message);
        self.create_log(&log).await
    }

    /// Méthode utilitaire pour log d'erreur
    pub async fn log_error(&self, log_type: &str, message: &str, context: Option<&str>) -> Result<Log> {
        let log = Log::error(log_type, message, context);
        self.create_log(&log).await
    }

    /// Méthode utilitaire pour log de debug
    pub async fn log_debug(&self, log_type: &str, message: &str, context: Option<&str>) -> Result<Log> {
        let log = Log::debug(log_type, message, context);
        self.create_log(&log).await
    }

    /// Méthode utilitaire pour log de warning
    pub async fn log_warn(&self, log_type: &str, message: &str, context: Option<&str>) -> Result<Log> {
        let log = Log::warn(log_type, message, context);
        self.create_log(&log).await
    }

    /// Récupère un log par ID
    pub async fn get_log_by_id(&self, id: Uuid) -> Result<Option<Log>> {
        let query = format!("SELECT * FROM logs WHERE id = '{}'", id);
        match self.db.run_query_fetch_optional(&query).await? {
            Some(row) => {
                let log = Log {
                    id: row.get("id"),
                    r#type: row.get("type"),
                    level: row.get("level"),
                    message: row.get("message"),
                    context: row.get("context"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(Some(log))
            },
            None => Ok(None)
        }
    }

    /// Récupère tous les logs (sans filtrage)
    pub async fn get_all_logs(&self) -> Result<Vec<Log>> {
        let query = "SELECT * FROM logs ORDER BY created_at DESC";
        let rows = self.db.run_query_fetch_all(query).await?;
        
        let mut logs = Vec::new();
        for row in rows {
            let log = Log {
                id: row.get("id"),
                r#type: row.get("type"),
                level: row.get("level"),
                message: row.get("message"),
                context: row.get("context"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            logs.push(log);
        }
        Ok(logs)
    }

    /// Récupère les logs par type
    pub async fn get_logs_by_type(&self, log_type: &str) -> Result<Vec<Log>> {
        let query = format!("SELECT * FROM logs WHERE type = '{}' ORDER BY created_at DESC", log_type);
        let rows = self.db.run_query_fetch_all(&query).await?;
        
        let mut logs = Vec::new();
        for row in rows {
            let log = Log {
                id: row.get("id"),
                r#type: row.get("type"),
                level: row.get("level"),
                message: row.get("message"),
                context: row.get("context"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            logs.push(log);
        }
        Ok(logs)
    }

    /// Récupère les logs par niveau
    pub async fn get_logs_by_level(&self, level: LogLevel) -> Result<Vec<Log>> {
        let query = format!("SELECT * FROM logs WHERE level = {} ORDER BY created_at DESC", level.as_i32());
        let rows = self.db.run_query_fetch_all(&query).await?;
        
        let mut logs = Vec::new();
        for row in rows {
            let log = Log {
                id: row.get("id"),
                r#type: row.get("type"),
                level: row.get("level"),
                message: row.get("message"),
                context: row.get("context"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            logs.push(log);
        }
        Ok(logs)
    }

    /// Supprime un log par ID
    pub async fn delete_log(&self, id: Uuid) -> Result<bool> {
        let query = format!("DELETE FROM logs WHERE id = '{}'", id);
        match self.db.run_query(&query).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false)
        }
    }

    /// Supprime les logs plus anciens qu'une date donnée
    pub async fn cleanup_old_logs(&self, before_date: OffsetDateTime) -> Result<u64> {
        let query = format!("DELETE FROM logs WHERE created_at < '{}'", before_date);
        self.db.run_query(&query).await?;
        Ok(1) // retourne 1 pour simuler le nombre de lignes supprimées
    }

    /// Compte le nombre de logs par type
    pub async fn count_logs_by_type(&self, log_type: &str) -> Result<i64> {
        let query = format!("SELECT COUNT(*) as count FROM logs WHERE type = '{}'", log_type);
        let row = self.db.run_query_fetch_one(&query).await?;
        let count: i64 = row.get("count");
        Ok(count)
    }

    /// Compte le nombre total de logs
    pub async fn count_all_logs(&self) -> Result<i64> {
        let query = "SELECT COUNT(*) as count FROM logs";
        let row = self.db.run_query_fetch_one(query).await?;
        let count: i64 = row.get("count");
        Ok(count)
    }
}
