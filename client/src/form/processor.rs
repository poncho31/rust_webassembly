use web_sys::{FormData, File};
use serde_json::{Value, json};
use crate::form::{FormField, FormConfig};
use wasm_bindgen::JsValue;
use std::collections::HashMap;

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
        let mut total_file_size = 0u64;

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
                                    total_file_size += file_size;
                                    
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
                    data[field.id()] = match field.field_type() {
                        crate::form::FieldType::Number => {
                            value.parse::<f64>()
                                .map(|n| json!(n))
                                .unwrap_or_else(|_| json!(value))
                        }
                        crate::form::FieldType::Checkbox => {
                            json!(field.input().checked())
                        }
                        _ => json!(value)
                    };
                }
            } else if field.has_files() {
                // For files, we'll include metadata only
                if let Some(files) = field.files() {
                    let mut file_names = Vec::new();
                    for i in 0..files.length() {
                        if let Some(file) = files.item(i) {
                            file_names.push(file.name());
                        }
                    }
                    data[field.id()] = json!(file_names);
                }
            }
        }
        
        Ok(data)
    }
}
