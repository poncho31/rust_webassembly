pub mod http_models;
pub mod errors;
pub mod config;
pub mod table;

// Database modules only available with database feature
#[cfg(feature = "database")]
pub mod db_models;
#[cfg(feature = "database")]
pub mod database;
#[cfg(feature = "database")]
pub mod database_repository;
#[cfg(feature = "database")]
pub mod repositories;

// Always available exports
pub use http_models::http_responses::HttpSendResponse;
pub use errors::{AppError, AppResult, AppErrorType};
pub use config::AppConfig;
pub use table::Table;

// Database exports only available with database feature
#[cfg(feature = "database")]
pub use db_models::user::User;
#[cfg(feature = "database")]
pub use db_models::form_data::{FormData, NewFormData};
#[cfg(feature = "database")]
pub use database::{create_database, init_db};
#[cfg(feature = "database")]
pub use database_repository::DatabaseRepository;