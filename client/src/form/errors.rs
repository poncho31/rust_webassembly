use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::fmt;

/// Application-specific error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormError {
    ValidationError(String),
    NetworkError(String),
    FileError(String),
    ConfigurationError(String),
    DOMError(String),
    Unknown(String),
}

impl fmt::Display for FormError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            FormError::NetworkError(msg) => write!(f, "Network Error: {}", msg),
            FormError::FileError(msg) => write!(f, "File Error: {}", msg),
            FormError::ConfigurationError(msg) => write!(f, "Configuration Error: {}", msg),
            FormError::DOMError(msg) => write!(f, "DOM Error: {}", msg),
            FormError::Unknown(msg) => write!(f, "Unknown Error: {}", msg),
        }
    }
}

impl From<JsValue> for FormError {
    fn from(js_val: JsValue) -> Self {
        let error_msg = js_val
            .as_string()
            .unwrap_or_else(|| format!("{:?}", js_val));
        
        // Try to categorize the error based on the message
        if error_msg.contains("network") || error_msg.contains("fetch") {
            FormError::NetworkError(error_msg)
        } else if error_msg.contains("validation") {
            FormError::ValidationError(error_msg)
        } else if error_msg.contains("file") {
            FormError::FileError(error_msg)
        } else if error_msg.contains("not found") || error_msg.contains("element") {
            FormError::DOMError(error_msg)
        } else {
            FormError::Unknown(error_msg)
        }
    }
}

impl Into<JsValue> for FormError {
    fn into(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
}

/// Result type for form operations
pub type FormResult<T> = Result<T, FormError>;

/// Error handler trait for customizable error handling
pub trait ErrorHandler {
    fn handle_error(&self, error: &FormError) -> bool;
}

/// Default error handler that logs errors
pub struct DefaultErrorHandler;

impl ErrorHandler for DefaultErrorHandler {
    fn handle_error(&self, error: &FormError) -> bool {
        crate::client_tools::log(&format!("Error handled: {}", error));
        false // Don't suppress the error
    }
}

/// Error context for better error reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub field: Option<String>,
    pub additional_info: Vec<String>,
}

impl ErrorContext {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            field: None,
            additional_info: Vec::new(),
        }
    }

    pub fn with_field(mut self, field: &str) -> Self {
        self.field = Some(field.to_string());
        self
    }

    pub fn with_info<S: Into<String>>(mut self, info: S) -> Self {
        self.additional_info.push(info.into());
        self
    }
}

/// Enhanced error with context
#[derive(Debug, Clone)]
pub struct ContextualError {
    pub error: FormError,
    pub context: ErrorContext,
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} during {}", self.error, self.context.operation)?;
        
        if let Some(field) = &self.context.field {
            write!(f, " (field: {})", field)?;
        }
        
        if !self.context.additional_info.is_empty() {
            write!(f, " [{}]", self.context.additional_info.join(", "))?;
        }
        
        Ok(())
    }
}

/// Utility functions for error handling
pub fn handle_js_error(js_error: JsValue, context: ErrorContext) -> ContextualError {
    let form_error = FormError::from(js_error);
    ContextualError {
        error: form_error,
        context,
    }
}

pub fn validation_error(message: &str, field: Option<&str>) -> FormError {
    let msg = if let Some(field) = field {
        format!("{} (field: {})", message, field)
    } else {
        message.to_string()
    };
    FormError::ValidationError(msg)
}

pub fn network_error(message: &str) -> FormError {
    FormError::NetworkError(message.to_string())
}

pub fn file_error(message: &str) -> FormError {
    FormError::FileError(message.to_string())
}

pub fn dom_error(message: &str) -> FormError {
    FormError::DOMError(message.to_string())
}
