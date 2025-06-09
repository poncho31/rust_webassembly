// Re-export the new modular form system and utility functions
use crate::form;

// Re-export main types and functions for convenience
pub use form::{
    FormConfig, 
    FormField, 
    FieldType,
    FieldConfig,
    FieldOption,
    FormHandler,
    FormValidator,
    ValidationRule,
    form_init,
    form_init_with_config,
};

// Additional utility functions for backward compatibility
use wasm_bindgen::prelude::*;
use std::collections::HashMap;

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
        .success_message("Message sent successfully!")
        .build();
    
    let validator = form::FormValidator::new()
        .add_rule("firstname", form::ValidationRule::text(2, 50))
        .add_rule("lastname", form::ValidationRule::text(2, 50))
        .add_rule("email", form::ValidationRule::email())
        .add_rule("message", form::ValidationRule::text(10, 1000));
    
    let handler = FormHandler::new(form_id, endpoint, config)
        .with_field_specs(fields)
        .build()?
        .with_validator(validator);
    
    handler.initialize()
}

/// Initialize an upload form with file validation
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
    
    let validator = form::FormValidator::new()
        .add_rule("title", form::ValidationRule::text(3, 100))
        .add_rule("description", form::ValidationRule {
            required: false,
            max_length: Some(500),
            ..Default::default()
        });
    
    let handler = FormHandler::new(form_id, endpoint, config)
        .with_field_specs(fields)
        .build()?
        .with_validator(validator);
    
    handler.initialize()
}

/// Initialize a registration form with comprehensive validation
pub fn registration_form_init(form_id: &str, endpoint: &str) -> Result<(), JsValue> {
    let fields = &[
        ("username", FieldType::Text),
        ("email", FieldType::Email),
        ("password", FieldType::Password),
        ("confirm_password", FieldType::Password),
        ("age", FieldType::Number),
        ("terms", FieldType::Checkbox),
    ];
    
    let config = FormConfig::builder()
        .validation(true)
        .auto_focus_error(true)
        .success_message("Registration successful!")
        .build();
    
    let validator = form::FormValidator::new()
        .add_rule("username", form::ValidationRule::text(3, 20))
        .add_rule("email", form::ValidationRule::email())
        .add_rule("password", form::ValidationRule::text(8, 100))
        .add_rule("age", form::ValidationRule::number(13.0, 120.0))
        .add_rule("terms", form::ValidationRule::required());
    
    let handler = FormHandler::new(form_id, endpoint, config)
        .with_field_specs(fields)
        .build()?
        .with_validator(validator);
    
    handler.initialize()
}

/// Create a form configuration builder for common scenarios
pub fn create_form_config(scenario: &str) -> FormConfig {
    match scenario {
        "contact" => FormConfig::builder()
            .validation(true)
            .success_message("Message envoyé avec succès!")
            .build(),
        "upload" => FormConfig::builder()
            .validation(true)
            .max_file_size(10 * 1024 * 1024)
            .success_message("Fichiers téléchargés avec succès!")
            .build(),
        "registration" => FormConfig::builder()
            .validation(true)
            .auto_focus_error(true)
            .success_message("Inscription réussie!")
            .build(),
        _ => FormConfig::default(),
    }
}

/// Helper function to create field configurations with common patterns
pub fn create_field_configs() -> HashMap<&'static str, FieldConfig> {
    let mut configs = HashMap::new();
    
    // Common field configurations
    configs.insert("email", FieldConfig::new(FieldType::Email)
        .with_placeholder("votre@email.com")
        .required());
    
    configs.insert("password", FieldConfig::new(FieldType::Password)
        .with_placeholder("Mot de passe")
        .required());
    
    configs.insert("firstname", FieldConfig::new(FieldType::Text)
        .with_placeholder("Prénom")
        .required());
    
    configs.insert("lastname", FieldConfig::new(FieldType::Text)
        .with_placeholder("Nom")
        .required());
    
    configs.insert("age", FieldConfig::new(FieldType::Number)
        .with_placeholder("Âge")
        .required());
    
    configs.insert("message", FieldConfig::new(FieldType::TextArea)
        .with_placeholder("Votre message"));    configs.insert("files", FieldConfig::new(FieldType::File));
    
    configs
}
