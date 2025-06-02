// Examples of how to use the improved form system
use wasm_bindgen::prelude::*;
use crate::form::{FormConfig, FormHandler, FieldType};
use crate::validation::{FormValidator, ValidationRule};

/// Example 1: Simple contact form
pub fn setup_contact_form() -> Result<(), JsValue> {
    let fields = &[
        ("name", FieldType::Text),
        ("email", FieldType::Email),
        ("subject", FieldType::Text),
        ("message", FieldType::TextArea),
    ];

    let config = FormConfig::builder()
        .validation(true)
        .auto_focus_error(true)
        .success_message("Thank you for your message!")
        .build();

    let validator = FormValidator::new()
        .add_rule("name", ValidationRule::text(2, 100))
        .add_rule("email", ValidationRule::email())
        .add_rule("subject", ValidationRule::text(5, 200))
        .add_rule("message", ValidationRule::text(10, 1000));

    let handler = FormHandler::new("contact-form", "/api/contact", Some(fields), config)?
        .with_validator(validator);

    handler.initialize()
}

/// Example 2: File upload form with size limits
pub fn setup_upload_form() -> Result<(), JsValue> {
    let fields = &[
        ("title", FieldType::Text),
        ("description", FieldType::TextArea),
        ("category", FieldType::Select),
        ("files", FieldType::File),
        ("public", FieldType::Checkbox),
    ];

    let config = FormConfig::builder()
        .validation(true)
        .max_file_size(10 * 1024 * 1024) // 10MB
        .success_message("Files uploaded successfully!")
        .retry_attempts(2)
        .build();

    let validator = FormValidator::new()
        .add_rule("title", ValidationRule::text(3, 100))
        .add_rule("description", ValidationRule {
            required: false,
            max_length: Some(500),
            ..Default::default()
        });

    let handler = FormHandler::new("upload-form", "/api/upload", Some(fields), config)?
        .with_validator(validator);

    handler.initialize()
}

/// Example 3: User registration form with complex validation
pub fn setup_registration_form() -> Result<(), JsValue> {
    let fields = &[
        ("username", FieldType::Text),
        ("email", FieldType::Email),
        ("password", FieldType::Password),
        ("confirm_password", FieldType::Password),
        ("birthdate", FieldType::Date),
        ("phone", FieldType::Tel),
        ("website", FieldType::Url),
        ("age", FieldType::Number),
        ("terms", FieldType::Checkbox),
    ];

    let config = FormConfig::builder()
        .validation(true)
        .auto_focus_error(true)
        .debounce_ms(500)
        .success_message("Registration successful! Please check your email.")
        .error_message("Registration failed. Please try again.")
        .build();

    let validator = FormValidator::new()
        .add_rule("username", ValidationRule::text(3, 30))
        .add_rule("email", ValidationRule::email())
        .add_rule("password", ValidationRule::text(8, 128))
        .add_rule("confirm_password", ValidationRule::text(8, 128))
        .add_rule("phone", ValidationRule {
            required: true,
            pattern: Some("phone".to_string()),
            ..Default::default()
        })
        .add_rule("website", ValidationRule {
            required: false,
            pattern: Some("url".to_string()),
            ..Default::default()
        })
        .add_rule("age", ValidationRule::number(13.0, 120.0))
        .add_rule("terms", ValidationRule::required());

    let handler = FormHandler::new("registration-form", "/api/register", Some(fields), config)?
        .with_validator(validator);

    handler.initialize()
}

/// Example 4: Survey form with optional fields
pub fn setup_survey_form() -> Result<(), JsValue> {
    let fields = &[
        ("satisfaction", FieldType::Radio),
        ("comments", FieldType::TextArea),
        ("recommend", FieldType::Checkbox),
        ("contact_me", FieldType::Checkbox),
        ("follow_up_email", FieldType::Email),
    ];

    let config = FormConfig::builder()
        .validation(true)
        .auto_focus_error(false) // Don't auto-focus for surveys
        .success_message("Thank you for your feedback!")
        .build();

    let validator = FormValidator::new()
        .add_rule("satisfaction", ValidationRule::required())
        .add_rule("comments", ValidationRule {
            required: false,
            max_length: Some(1000),
            ..Default::default()
        })
        .add_rule("follow_up_email", ValidationRule {
            required: false,
            pattern: Some("email".to_string()),
            ..Default::default()
        });

    let handler = FormHandler::new("survey-form", "/api/survey", Some(fields), config)?
        .with_validator(validator);

    handler.initialize()
}

/// Example 5: Quick form setup using convenience functions
pub fn setup_quick_forms() -> Result<(), JsValue> {
    // Use the convenience functions from client_form_improved.rs
    crate::client_form_improved::contact_form_init("contact", "/api/contact")?;
    crate::client_form_improved::upload_form_init("upload", "/api/upload")?;
    crate::client_form_improved::registration_form_init("register", "/api/register")?;
    
    Ok(())
}

/// Example 6: Custom validation with business logic
pub fn setup_custom_validation_form() -> Result<(), JsValue> {
    let fields = &[
        ("start_date", FieldType::Date),
        ("end_date", FieldType::Date),
        ("budget", FieldType::Number),
        ("project_type", FieldType::Select),
    ];

    let config = FormConfig::builder()
        .validation(true)
        .success_message("Project created successfully!")
        .build();

    // Custom validator with business rules
    let validator = FormValidator::new()
        .add_rule("start_date", ValidationRule::required())
        .add_rule("end_date", ValidationRule::required())
        .add_rule("budget", ValidationRule::number(100.0, 1000000.0))
        .add_rule("project_type", ValidationRule::required());

    let handler = FormHandler::new("project-form", "/api/projects", Some(fields), config)?
        .with_validator(validator);

    handler.initialize()
}

/// Example 7: Multi-step form (conceptual)
pub fn setup_multistep_form() -> Result<(), JsValue> {
    // Step 1: Personal Info
    let step1_fields = &[
        ("first_name", FieldType::Text),
        ("last_name", FieldType::Text),
        ("email", FieldType::Email),
    ];

    let step1_config = FormConfig::builder()
        .validation(true)
        .success_message("Step 1 completed!")
        .build();

    let step1_validator = FormValidator::new()
        .add_rule("first_name", ValidationRule::text(2, 50))
        .add_rule("last_name", ValidationRule::text(2, 50))
        .add_rule("email", ValidationRule::email());

    let step1_handler = FormHandler::new("step1-form", "/api/step1", Some(step1_fields), step1_config)?
        .with_validator(step1_validator);

    step1_handler.initialize()?;

    // Step 2: Additional Info
    let step2_fields = &[
        ("company", FieldType::Text),
        ("position", FieldType::Text),
        ("experience", FieldType::Number),
    ];

    let step2_config = FormConfig::builder()
        .validation(true)
        .success_message("Registration completed!")
        .build();

    let step2_validator = FormValidator::new()
        .add_rule("company", ValidationRule::text(2, 100))
        .add_rule("position", ValidationRule::text(2, 100))
        .add_rule("experience", ValidationRule::number(0.0, 50.0));

    let step2_handler = FormHandler::new("step2-form", "/api/step2", Some(step2_fields), step2_config)?
        .with_validator(step2_validator);

    step2_handler.initialize()?;

    Ok(())
}
