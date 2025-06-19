// Library interface for Android builds
// This file provides a C-compatible interface for the Rust server

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::thread;
use std::time::Duration;

// Re-export existing modules
pub mod extract_form;
pub mod models;

// For Android JNI compatibility, we provide a simplified interface
/// Initialize the Rust server
/// Returns 0 on success, -1 on error
#[no_mangle]
pub extern "C" fn rust_server_init() -> i32 {
    env_logger::try_init().unwrap_or(());
    println!("Rust server library initialized");
    0
}

/// Start the server on the specified port
/// Returns 0 on success, -1 on error
#[no_mangle]
pub extern "C" fn rust_server_start(port: u16) -> i32 {
    println!("Starting Rust server on port {}", port);
    // For now, just simulate server start
    // In a full implementation, this would start the actual Actix web server
    0
}

/// Stop the server
/// Returns 0 on success, -1 on error
#[no_mangle]
pub extern "C" fn rust_server_stop() -> i32 {
    println!("Stopping Rust server");
    0
}

/// Cleanup resources
#[no_mangle]
pub extern "C" fn rust_server_cleanup() {
    println!("Rust server cleanup complete");
}

/// Get server status
/// Returns 1 if running, 0 if stopped
#[no_mangle]
pub extern "C" fn rust_server_status() -> i32 {
    // For now, always return running status
    1
}

/// Test function to verify the library is working
#[no_mangle]
pub extern "C" fn rust_server_test() -> *const c_char {
    let response = CString::new("Rust server library is working!").unwrap();
    response.into_raw()
}

/// Free a string returned by the library
#[no_mangle]
pub extern "C" fn rust_server_free_string(s: *mut c_char) {
    unsafe {
        if !s.is_null() {
            CString::from_raw(s);
        }
    }
}