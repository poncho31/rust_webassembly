// Form module - Better organization
pub mod config;
pub mod field;
pub mod handler;
pub mod processor;
pub mod errors;
pub mod cache;
pub mod examples;

pub use config::FormConfig;
pub use field::{FormField, FieldType, FieldConfig, FieldOption};
pub use handler::{FormHandler, form_init, form_init_with_config};
pub use processor::FormProcessor;
pub use errors::{FormError, FormResult, ErrorContext};
