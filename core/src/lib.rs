pub mod http_models;
pub mod config;
pub mod table;

#[cfg(feature = "database")]
pub mod database;

#[cfg(feature = "database")]
pub mod repositories;

// Always available exports
pub use http_models::http_responses::HttpSendResponse;
pub use table::Table;

#[cfg(feature = "database")]
pub use database::{init_db};

#[cfg(feature = "database")]
pub use repositories::{UserRepository};