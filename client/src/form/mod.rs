// Form module - Refactored and consolidated organization
pub mod form_config;
pub mod form_field;
pub mod form_handler;
pub mod form_core;
pub mod form_validation;

// Core exports
pub use form_config::FormConfig;
pub use form_field::{FormField, FieldType, FieldConfig, FieldOption};
pub use form_handler::{FormHandler, form_init, form_init_with_config};
pub use form_core::{FormProcessor, FormError, FormResult, ErrorContext};
pub use form_validation::{FormValidator, ValidationRule};