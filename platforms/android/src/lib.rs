use jni::JNIEnv;
use jni::objects::{JClass, JString, JObject};
use jni::sys::{jstring, jboolean};
use android_logger::{Config, FilterBuilder};
use log::{info, error, warn};
use std::sync::{Once, Arc, Mutex};
use tokio::runtime::Runtime;
use std::thread;

// Import du serveur existant
use server_lib::{start_full_web_server, WebServerConfig};

// Modules locaux
mod android_config;

use android_config::AndroidServerConfig;

static INIT: Once = Once::new();
static mut SERVER_RUNTIME: Option<Arc<Mutex<Runtime>>> = None;
static mut SERVER_HANDLE: Option<tokio::task::JoinHandle<()>> = None;

/// Initialize logging for Android
fn init_logging() {
    INIT.call_once(|| {
        android_logger::init_once(
            Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("rust_webassembly_android")
        );
        info!("Android logging initialized");
    });
}

/// Initialize the Rust backend and start the embedded server
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_initRust(
    env: JNIEnv,
    _class: JClass,
) -> jboolean {
    init_logging();
    info!("Initializing Rust backend for Android");
    
    match start_embedded_server() {
        Ok(_) => {
            info!("Embedded server started successfully");
            1 // true
        },
        Err(e) => {
            error!("Failed to start embedded server: {}", e);
            0 // false
        }
    }
}

/// Start the embedded server using the existing server code
fn start_embedded_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    unsafe {
        if SERVER_RUNTIME.is_some() {
            warn!("Server already running");
            return Ok(());
        }
        
        // Configuration Android
        let android_config = AndroidServerConfig::new();
        
        // Créer les répertoires nécessaires
        if let Err(e) = android_config.create_directories() {
            warn!("Could not create Android directories: {}", e);
        }
        
        // Configurer l'environnement pour Android
        android_config.setup_android_environment();
        
        // Créer un runtime Tokio pour Android
        let rt = Arc::new(Mutex::new(
            Runtime::new().map_err(|e| format!("Failed to create Tokio runtime: {}", e))?
        ));
        
        let rt_clone = rt.clone();
        
        // Démarrer le serveur dans un thread séparé
        let server_handle = thread::spawn(move || {
            let rt_guard = rt_clone.lock().unwrap();
            rt_guard.block_on(async {
                info!("Starting embedded web server for Android...");
                
                // Utiliser la fonction start_full_web_server existante
                match start_full_web_server().await {
                    Ok(_) => {
                        info!("Server stopped gracefully");
                    },
                    Err(e) => {
                        error!("Server error: {}", e);
                    }
                }
            });
        });
        
        SERVER_RUNTIME = Some(rt);
        
        // Attendre un peu pour que le serveur démarre
        thread::sleep(std::time::Duration::from_millis(1000));
        
        Ok(())
    }
}

/// Get the server URL that the WebView should load
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_getServerUrl(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    init_logging();
    
    // URL du serveur local pour Android
    let server_url = "http://127.0.0.1:8080";
    
    info!("Providing server URL: {}", server_url);
    
    let output = env.new_string(server_url)
        .expect("Couldn't create java string!");
    
    output.into_raw()
}

/// Handle WebView messages from JavaScript
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_handleWebViewMessage(
    mut env: JNIEnv,
    _class: JClass,
    message: JString,
) -> jstring {
    init_logging();
    
    let message_str: String = env.get_string(&message)
        .expect("Couldn't get java string!")
        .into();
    
    info!("Received message from WebView: {}", message_str);
    
    // Process the message and return a response
    let response = process_client_message(&message_str);
    
    let output = env.new_string(response)
        .expect("Couldn't create java string!");
    
    output.into_raw()
}

/// Process messages from the client (similar to your existing client logic)
fn process_client_message(message: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(message) {
        Ok(json_message) => {
            info!("Processing JSON message: {:?}", json_message);
            
            // Ici on peut intégrer avec la logique core existante
            // ou faire des appels HTTP vers le serveur local
            
            serde_json::json!({
                "status": "success",
                "message": "Message processed successfully",
                "echo": json_message,
                "server_url": "http://127.0.0.1:8080"
            }).to_string()
        }
        Err(_) => {
            error!("Failed to parse message as JSON: {}", message);
            serde_json::json!({
                "status": "error",
                "message": "Invalid JSON format"
            }).to_string()
        }
    }
}

/// Check if embedded server is running
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_isServerRunning(
    env: JNIEnv,
    _class: JClass,
) -> jboolean {
    init_logging();
    
    unsafe {
        let is_running = SERVER_RUNTIME.is_some();
        info!("Server running status: {}", is_running);
        if is_running { 1 } else { 0 }
    }
}

/// Stop the embedded server
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_stopServer(
    env: JNIEnv,
    _class: JClass,
) -> jboolean {
    init_logging();
    info!("Stopping embedded server...");
    
    unsafe {
        if let Some(runtime) = SERVER_RUNTIME.take() {
            // Le serveur s'arrêtera quand le runtime sera droppé
            drop(runtime);
            info!("Server stopped successfully");
            1 // true
        } else {
            warn!("Server was not running");
            0 // false
        }
    }
}

/// Get server status information
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_getServerStatus(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    init_logging();
    
    unsafe {
        let status = if SERVER_RUNTIME.is_some() {
            serde_json::json!({
                "status": "running",
                "url": "http://127.0.0.1:8080",
                "environment": "android",
                "ssl_enabled": false
            })
        } else {
            serde_json::json!({
                "status": "stopped",
                "message": "Server is not running"
            })
        };
        
        let status_str = status.to_string();
        info!("Server status: {}", status_str);
        
        let output = env.new_string(status_str)
            .expect("Couldn't create java string!");
        
        output.into_raw()
    }
}
