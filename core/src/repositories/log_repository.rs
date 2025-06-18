use sqlx::{PgPool, Row, FromRow};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, FromRow)]
pub struct Log {
    pub id: Uuid,
    pub r#type: String,        // type est un mot-clé réservé, on utilise r#type
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
    pool: PgPool,
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
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Crée un nouveau log
    pub async fn create_log(&self, log: &Log) -> Result<Log> {
        let result = sqlx::query_as::<_, Log>(
            r#"INSERT INTO logs 
                    (id, type, level, message, context, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING *"#
        )
        .bind(&log.id)
        .bind(&log.r#type)
        .bind(&log.level)
        .bind(&log.message)
        .bind(&log.context)
        .bind(&log.created_at)
        .bind(&log.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
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
        let result = sqlx::query_as::<_, Log>(
            "SELECT * FROM logs WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }    /// Récupère tous les logs avec filtrage optionnel
    pub async fn get_logs(
        &self,
        log_type: Option<&str>,
        level: Option<LogLevel>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Log>> {
        let mut query = "SELECT * FROM logs WHERE 1=1".to_string();
        let mut bind_params: Vec<String> = Vec::new();

        if let Some(log_type) = log_type {
            bind_params.push("type".to_string());
            query.push_str(&format!(" AND type = ${}", bind_params.len()));
        }

        if let Some(level) = level {
            bind_params.push("level".to_string());
            query.push_str(&format!(" AND level = ${}", bind_params.len()));
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(_limit) = limit {
            bind_params.push("limit".to_string());
            query.push_str(&format!(" LIMIT ${}", bind_params.len()));
        }

        if let Some(_offset) = offset {
            bind_params.push("offset".to_string());
            query.push_str(&format!(" OFFSET ${}", bind_params.len()));
        }

        let mut query_builder = sqlx::query_as::<_, Log>(&query);

        if let Some(log_type) = log_type {
            query_builder = query_builder.bind(log_type);
        }

        if let Some(level) = level {
            query_builder = query_builder.bind(level.as_i32());
        }

        if let Some(limit) = limit {
            query_builder = query_builder.bind(limit);
        }

        if let Some(offset) = offset {
            query_builder = query_builder.bind(offset);
        }

        let results = query_builder
            .fetch_all(&self.pool)
            .await?;

        Ok(results)
    }

    /// Récupère tous les logs (sans filtrage)
    pub async fn get_all_logs(&self) -> Result<Vec<Log>> {
        let results = sqlx::query_as::<_, Log>(
            "SELECT * FROM logs ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Récupère les logs par type
    pub async fn get_logs_by_type(&self, log_type: &str) -> Result<Vec<Log>> {
        let results = sqlx::query_as::<_, Log>(
            "SELECT * FROM logs WHERE type = $1 ORDER BY created_at DESC"
        )
        .bind(log_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Récupère les logs par niveau
    pub async fn get_logs_by_level(&self, level: LogLevel) -> Result<Vec<Log>> {
        let results = sqlx::query_as::<_, Log>(
            "SELECT * FROM logs WHERE level = $1 ORDER BY created_at DESC"
        )
        .bind(level.as_i32())
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Supprime un log par ID
    pub async fn delete_log(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM logs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Supprime les logs plus anciens qu'une date donnée
    pub async fn cleanup_old_logs(&self, before_date: OffsetDateTime) -> Result<u64> {
        let result = sqlx::query("DELETE FROM logs WHERE created_at < $1")
            .bind(before_date)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Compte le nombre de logs par type
    pub async fn count_logs_by_type(&self, log_type: &str) -> Result<i64> {
        let result = sqlx::query("SELECT COUNT(*) as count FROM logs WHERE type = $1")
            .bind(log_type)
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = result.get("count");
        Ok(count)
    }

    /// Compte le nombre total de logs
    pub async fn count_all_logs(&self) -> Result<i64> {
        let result = sqlx::query("SELECT COUNT(*) as count FROM logs")
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = result.get("count");
        Ok(count)
    }
}
