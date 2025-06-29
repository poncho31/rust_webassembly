use jni::JNIEnv;
use jni::objects::{JClass, JString, JObject};
use jni::sys::{jstring, jboolean};
use android_logger::{Config, FilterBuilder};
use log::{info, error, warn};
use std::sync::{Once, Arc, Mutex, mpsc};
use std::time::Duration;
use tokio::runtime::Runtime;
use std::thread;
use std::net::TcpStream;

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
        
        // Créer un canal pour recevoir le statut du serveur
        let (tx, rx) = mpsc::channel::<Result<String, String>>();
        
        // Démarrer le serveur dans un thread séparé
        let _server_handle = thread::spawn(move || {
            let rt_guard = rt_clone.lock().unwrap();
            rt_guard.block_on(async {
                info!("Starting embedded web server for Android...");
                
                // Log les variables d'environnement importantes
                info!("SERVER_HOST: {}", std::env::var("SERVER_HOST").unwrap_or("NOT_SET".to_string()));
                info!("SERVER_PORT: {}", std::env::var("SERVER_PORT").unwrap_or("NOT_SET".to_string()));
                info!("SSL_ENABLED: {}", std::env::var("SSL_ENABLED").unwrap_or("NOT_SET".to_string()));
                info!("DATABASE_URL: {}", std::env::var("DATABASE_URL").unwrap_or("NOT_SET".to_string()));
                info!("ENVIRONMENT: {}", std::env::var("ENVIRONMENT").unwrap_or("NOT_SET".to_string()));
                
                // Utiliser le serveur complet comme demandé
                info!("Starting full web server for Android...");
                
                // Test la connectivité au port avant de démarrer
                let host = std::env::var("SERVER_HOST").unwrap_or("127.0.0.1".to_string());
                let port = std::env::var("SERVER_PORT").unwrap_or("8088".to_string()).parse::<u16>().unwrap_or(8088);
                
                // Vérifier si le port est libre
                if let Ok(listener) = std::net::TcpListener::bind((host.clone(), port)) {
                    info!("Port {}:{} is available for binding", host, port);
                    drop(listener); // Libérer le port pour Actix
                } else {
                    let error_msg = format!("Port {}:{} is already in use or not available", host, port);
                    error!("{}", error_msg);
                    let _ = tx.send(Err(error_msg));
                    return;
                }
                
                // Tenter de démarrer le serveur
                match start_full_web_server().await {
                    Ok(_) => {
                        info!("Full web server stopped gracefully");
                        let _ = tx.send(Ok("Server started successfully".to_string()));
                    },
                    Err(e) => {
                        error!("Full web server startup failed: {}", e);
                        error!("Error kind: {:?}", e.kind());
                        error!("Error details: {:#?}", e);
                        
                        // Try to be more specific about the error
                        let error_string = format!("{}", e);
                        let detailed_error = if error_string.contains("database") || error_string.contains("Database") {
                            format!("Database connection issue: {} - check DATABASE_URL and SQLite setup", error_string)
                        } else if error_string.contains("bind") || error_string.contains("address") {
                            format!("Network binding issue: {} - check if port {} is available", error_string, port)
                        } else if error_string.contains("static") || error_string.contains("file") {
                            format!("Static files issue: {} - check if assets are accessible", error_string)
                        } else {
                            format!("Server startup error: {}", error_string)
                        };
                        
                        error!("{}", detailed_error);
                        let _ = tx.send(Err(detailed_error));
                    }
                }
            });
        });
        
        SERVER_RUNTIME = Some(rt);
        
        // Attendre un signal du serveur ou timeout
        info!("Waiting for server startup status...");
        match rx.recv_timeout(std::time::Duration::from_millis(5000)) {
            Ok(Ok(message)) => {
                info!("Server startup success: {}", message);
            },
            Ok(Err(error)) => {
                error!("Server failed to start: {}", error);
                return Err(format!("Server startup failed: {}", error).into());
            },
            Err(_) => {
                warn!("Server startup timeout - this might indicate a critical error");
                return Err("Server startup timeout - server may have failed to start".into());
            }
        }
        
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
    let server_url = "http://127.0.0.1:8088";
    
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
                "server_url": "http://127.0.0.1:8088"
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
                "url": "http://127.0.0.1:8088",
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

/// Test if server is actually listening on the expected port
fn test_server_connectivity(host: &str, port: u16) -> bool {
    match TcpStream::connect((host, port)) {
        Ok(_) => {
            info!("Successfully connected to server at {}:{}", host, port);
            true
        },
        Err(e) => {
            error!("Failed to connect to server at {}:{}: {}", host, port, e);
            false
        }
    }
}

/// Test server connectivity (for debugging)
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_testServerConnectivity(
    env: JNIEnv,
    _class: JClass,
) -> jboolean {
    init_logging();
    
    let host = std::env::var("SERVER_HOST").unwrap_or("127.0.0.1".to_string());
    let port = std::env::var("SERVER_PORT").unwrap_or("8088".to_string()).parse::<u16>().unwrap_or(8088);
    
    info!("Testing server connectivity to {}:{}", host, port);
    
    if test_server_connectivity(&host, port) {
        info!("Server connectivity test PASSED");
        1 // true
    } else {
        error!("Server connectivity test FAILED");
        0 // false
    }
}
