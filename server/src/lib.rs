// Library interface for Android builds and other integrations
// This file provides both C-compatible interface and Rust library exports

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Re-export existing modules for external use
pub mod extract_form;
pub mod models;
pub mod controllers;
pub mod ssl_config;

// Module contenant la logique complète du serveur
pub mod server_lib;

// Re-export the main server function and types from server_lib.rs
// This allows Android to use the existing server code without duplication
pub use server_lib::{start_full_web_server, WebServerConfig, create_web_server_config};