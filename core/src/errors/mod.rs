use serde::{Serialize, Deserialize};
use std::fmt;

/// Types d'erreurs standardisées pour l'application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppErrorType {
    ValidationError,
    DatabaseError,
    NetworkError,
    FileSystemError,
    AuthenticationError,
    SerializationError,
    UnknownError,
}

/// Structure d'erreur unifiée pour toute l'application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub error_type: AppErrorType,
    pub message: String,
    pub details: Option<String>,
    pub timestamp: Option<String>,
}

impl AppError {
    pub fn new(error_type: AppErrorType, message: impl Into<String>) -> Self {
        Self {
            error_type,
            message: message.into(),
            details: None,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(AppErrorType::ValidationError, message)
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::new(AppErrorType::DatabaseError, message)
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::new(AppErrorType::NetworkError, message)
    }

    pub fn filesystem(message: impl Into<String>) -> Self {
        Self::new(AppErrorType::FileSystemError, message)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for AppError {}

/// Result type personnalisé pour l'application
pub type AppResult<T> = Result<T, AppError>;
