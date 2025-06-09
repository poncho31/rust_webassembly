//! Core form processing and error handling functionality

use web_sys::{FormData, File};
use serde_json::{Value, json};
use crate::form::{FormField, FormConfig};
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;

// ============================================================================
// ERROR HANDLING
// ============================================================================

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
        } else if error_msg.contains("DOM") || error_msg.contains("element") {
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

// ============================================================================
// FORM PROCESSING
// ============================================================================

/// Handles form data processing and serialization
pub struct FormProcessor;

impl FormProcessor {
    /// Process form fields into FormData and debug information
    pub async fn process_fields(
        fields: &[FormField], 
        config: &FormConfig
    ) -> Result<(FormData, Value), JsValue> {
        let form_data = FormData::new()?;
        let mut debug_data = json!({});
        let mut _total_file_size = 0u64;

        for field in fields {
            match field.field_type() {
                crate::form::FieldType::File => {
                    if let Some(files) = field.files() {
                        let mut files_info = Vec::new();
                        
                        for i in 0..files.length() {
                            if let Some(file) = files.item(i) {
                                // Check file size if limit is set
                                if let Some(max_size) = config.max_file_size {
                                    let file_size = file.size() as u64;
                                    _total_file_size += file_size;
                                    
                                    if file_size > max_size {
                                        return Err(JsValue::from_str(&format!(
                                            "File {} exceeds maximum size of {} bytes",
                                            file.name(),
                                            max_size
                                        )));
                                    }
                                }

                                form_data.append_with_blob(field.id(), &file)?;
                                files_info.push(Self::file_info(&file));
                            }
                        }
                        debug_data[field.id()] = json!(files_info);
                    }
                }
                _ => {
                    let value = field.value();
                    if !value.is_empty() || field.is_required() {
                        form_data.append_with_str(field.id(), &value)?;
                        debug_data[field.id()] = json!(value);
                    }
                }
            }
        }

        Ok((form_data, debug_data))
    }

    /// Extract form values as a HashMap for validation
    pub fn extract_values(fields: &[FormField]) -> HashMap<String, String> {
        fields.iter()
            .filter(|field| !field.field_type().supports_files())
            .map(|field| (field.id().to_string(), field.value()))
            .collect()
    }

    /// Validate all form fields
    pub fn validate_fields(fields: &[FormField]) -> Result<(), Vec<String>> {
        let errors: Vec<String> = fields.iter()
            .filter_map(|field| field.validation_error())
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Focus on the first invalid field
    pub fn focus_first_error(fields: &[FormField]) -> Result<(), JsValue> {
        for field in fields {
            if !field.is_valid() {
                return field.focus();
            }
        }
        Ok(())
    }

    /// Clear all form fields
    pub fn clear_fields(fields: &[FormField]) -> Result<(), JsValue> {
        for field in fields {
            field.set_value("")?;
        }
        Ok(())
    }

    /// Create file information for debugging
    fn file_info(file: &File) -> serde_json::Value {
        json!({
            "name": file.name(),
            "size": file.size(),
            "type": file.type_(),
            "lastModified": file.last_modified()
        })
    }

    /// Serialize form data to JSON for API calls
    pub fn to_json(fields: &[FormField]) -> Result<Value, JsValue> {
        let mut data = json!({});
        
        for field in fields {
            if !field.field_type().supports_files() {
                let value = field.value();
                if !value.is_empty() {
                    data[field.id()] = json!(value);
                }
            } else if field.has_files() {
                // For files, we'll include metadata only
                if let Some(files) = field.files() {
                    let mut files_info = Vec::new();
                    for i in 0..files.length() {
                        if let Some(file) = files.item(i) {
                            files_info.push(Self::file_info(&file));
                        }
                    }
                    data[field.id()] = json!(files_info);
                }
            }
        }
        
        Ok(data)
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Handle JavaScript errors with context
pub fn handle_js_error(js_error: JsValue, _context: ErrorContext) -> FormError {
    let form_error = FormError::from(js_error);
    form_error
}

/// Create validation error
pub fn validation_error(message: &str, field: Option<&str>) -> FormError {
    let msg = if let Some(field) = field {
        format!("{} (field: {})", message, field)
    } else {
        message.to_string()
    };
    FormError::ValidationError(msg)
}

/// Create network error
pub fn network_error(message: &str) -> FormError {
    FormError::NetworkError(message.to_string())
}

/// Create file error
pub fn file_error(message: &str) -> FormError {
    FormError::FileError(message.to_string())
}

/// Create DOM error
pub fn dom_error(message: &str) -> FormError {
    FormError::DOMError(message.to_string())
}
