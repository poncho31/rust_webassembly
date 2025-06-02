// Re-export the new modular form system
use crate::form;

// Legacy compatibility - re-export main types and functions
pub use form::{
    FormConfig, 
    FormField, 
    FieldType,
    FormHandler,
    form_init,
    form_init_with_config,
};

// Additional utility functions for backward compatibility
use wasm_bindgen::prelude::*;

/// Quick form initialization with minimal configuration
pub fn quick_form_init(form_id: &str, endpoint: &str) -> Result<(), JsValue> {
    form_init(form_id, endpoint, None)
}

/// Initialize a contact form with predefined fields
pub fn contact_form_init(form_id: &str, endpoint: &str) -> Result<(), JsValue> {
    let fields = &[
        ("firstname", FieldType::Text),
        ("lastname", FieldType::Text),
        ("email", FieldType::Email),
        ("message", FieldType::TextArea),
    ];
    
    let config = FormConfig::builder()
        .validation(true)
        .auto_focus_error(true)
        .success_message("Thank you for your message! We'll get back to you soon.")
        .build();
    
    form_init_with_config(form_id, endpoint, Some(fields), config)
}

/// Initialize a file upload form
pub fn upload_form_init(form_id: &str, endpoint: &str) -> Result<(), JsValue> {
    let fields = &[
        ("title", FieldType::Text),
        ("description", FieldType::TextArea),
        ("files", FieldType::File),
    ];
    
    let config = FormConfig::builder()
        .validation(true)
        .max_file_size(10 * 1024 * 1024) // 10MB
        .success_message("Files uploaded successfully!")
        .build();
    
    form_init_with_config(form_id, endpoint, Some(fields), config)
}

/// Initialize a user registration form
pub fn registration_form_init(form_id: &str, endpoint: &str) -> Result<(), JsValue> {
    let fields = &[
        ("username", FieldType::Text),
        ("email", FieldType::Email),
        ("password", FieldType::Password),
        ("confirmPassword", FieldType::Password),
        ("birthdate", FieldType::Date),
        ("phone", FieldType::Tel),
    ];
    
    let config = FormConfig::builder()
        .validation(true)
        .auto_focus_error(true)
        .debounce_ms(500)
        .success_message("Registration successful! Please check your email.")
        .build();
    
    form_init_with_config(form_id, endpoint, Some(fields), config)
}
