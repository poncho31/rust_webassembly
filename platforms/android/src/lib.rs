use jni::JNIEnv;
use jni::objects::{JClass, JString, JObject};
use jni::sys::{jstring, jboolean};
use android_logger::{Config, FilterBuilder};
use log::{info, error};
use std::sync::Once;

static INIT: Once = Once::new();
// PAS ENCORE UTILISE, il faudra utiliser ceci quand je voudrais implémenter une partie serveur directement dans android
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

/// Initialize the Rust backend
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_initRust(
    env: JNIEnv,
    _class: JClass,
) -> jboolean {
    init_logging();
    info!("Initializing Rust backend for Android");
    
    // Initialize your backend here if needed
    // This could start a local server or prepare data
    
    1 // true
}

/// Get the server URL that the WebView should load
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_getServerUrl(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    init_logging();
    
    // Pour la version Android, nous utilisons une page HTML simple intégrée
    // ou un serveur local sur un port disponible
    let server_url = "http://10.0.2.2:8080"; // Adresse pour émulateur Android
    
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
    // This is where you can integrate with your existing Rust logic
    let response = process_client_message(&message_str);
    
    let output = env.new_string(response)
        .expect("Couldn't create java string!");
    
    output.into_raw()
}

/// Process messages from the client (similar to your existing client logic)
fn process_client_message(message: &str) -> String {
    // Parse the message and handle it using your existing core logic
    match serde_json::from_str::<serde_json::Value>(message) {
        Ok(json_message) => {
            info!("Processing JSON message: {:?}", json_message);
            
            // Here you can integrate with your existing core library
            // and return appropriate responses
            
            serde_json::json!({
                "status": "success",
                "message": "Message processed successfully",
                "echo": json_message
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

/// Start embedded server (optional - for offline mode)
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_startEmbeddedServer(
    env: JNIEnv,
    _class: JClass,
    port: i32,
) -> jboolean {
    init_logging();
    info!("Starting embedded server on port {}", port);
    
    // Pour cette démo, nous retournons une page HTML simple
    // Dans une implémentation complète, vous pourriez démarrer un serveur HTTP
    // en utilisant votre code serveur existant
    
    // Pour l'instant, nous simulons le démarrage réussi
    1 // true
}
