use jni::JNIEnv;
use jni::objects::{JClass};
use jni::sys::{jstring, jboolean};
use android_logger::Config;
use log::{info, error, warn};
use std::sync::{Once, Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use std::net::TcpStream;

// Import du serveur existant
use server_lib::start_full_web_server;

// Modules locaux
mod android_config;
use android_config::AndroidServerConfig;

static INIT: Once = Once::new();
static mut SERVER_RUNTIME: Option<Arc<Mutex<Runtime>>> = None;

/// Initialize logging for Android
fn init_logging() {
    INIT.call_once(|| {
        android_logger::init_once(
            Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("rust_webassembly_android")
        );
        info!("🚀 Android logging initialized - THIS SHOULD APPEAR IN LOGCAT");
    });
}

/// Initialize the Rust backend and start the embedded server
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_initRust(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    // PREMIER LOG ABSOLU - avant toute autre chose
    println!("=== RUST FUNCTION CALLED ===");
    eprintln!("=== RUST FUNCTION CALLED (stderr) ===");
    
    // Initialiser le logging en premier
    init_logging();
    
    info!("🚀 [INIT] initRust() called from Java");
    android_logger::init_once(
        Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("rust_webassembly_android")
    );
    
    info!("🚀 [INIT] Starting full server initialization...");
    
    // Wrap dans un catch_unwind pour capturer les panics
    let result = std::panic::catch_unwind(|| {
        start_embedded_server()
    });
    
    match result {
        Ok(Ok(())) => {
            info!("✅ [INIT] Server started successfully!");
            println!("✅ RUST SERVER STARTED SUCCESSFULLY");
            eprintln!("✅ RUST SERVER STARTED SUCCESSFULLY");
            1 // true
        },
        Ok(Err(e)) => {
            error!("❌ [INIT] Failed to start server: {}", e);
            println!("❌ RUST SERVER FAILED: {}", e);
            eprintln!("❌ RUST SERVER FAILED: {}", e);
            0 // false
        },
        Err(panic_info) => {
            let panic_msg = format!("PANIC in server initialization: {:?}", panic_info);
            error!("💥 [INIT] {}", panic_msg);
            println!("💥 {}", panic_msg);
            eprintln!("💥 {}", panic_msg);
            0 // false
        }
    }
}

/// Start the embedded server using start_full_web_server
fn start_embedded_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 [EMBEDDED] start_embedded_server() called - MUST APPEAR");
    eprintln!("🚀 [EMBEDDED] start_embedded_server() called - MUST APPEAR");
    info!("🚀 [EMBEDDED] start_embedded_server() called");
    
    unsafe {
        if SERVER_RUNTIME.is_some() {
            warn!("⚠️ [EMBEDDED] Server already running");
            return Ok(());
        }
        
        println!("📋 [EMBEDDED] Setting up Android configuration...");
        eprintln!("📋 [EMBEDDED] Setting up Android configuration...");
        info!("📋 [EMBEDDED] Setting up Android configuration...");
        
        // Configuration Android
        let android_config = match std::panic::catch_unwind(|| AndroidServerConfig::new()) {
            Ok(config) => {
                println!("✅ [EMBEDDED] AndroidServerConfig created successfully");
                info!("✅ [EMBEDDED] AndroidServerConfig created successfully");
                config
            },
            Err(_) => {
                println!("❌ [EMBEDDED] PANIC in AndroidServerConfig::new()");
                eprintln!("❌ [EMBEDDED] PANIC in AndroidServerConfig::new()");
                error!("❌ [EMBEDDED] PANIC in AndroidServerConfig::new()");
                return Err("Panic in AndroidServerConfig::new()".into());
            }
        };
        
        // Créer les répertoires nécessaires
        println!("📁 [EMBEDDED] Creating Android directories...");
        info!("📁 [EMBEDDED] Creating Android directories...");
        if let Err(e) = android_config.create_directories() {
            println!("⚠️ [EMBEDDED] Could not create Android directories: {}", e);
            warn!("⚠️ [EMBEDDED] Could not create Android directories: {}", e);
        } else {
            println!("✅ [EMBEDDED] Android directories created");
            info!("✅ [EMBEDDED] Android directories created");
        }
        
        // Configurer l'environnement pour Android
        println!("🔧 [EMBEDDED] Setting up Android environment...");
        info!("🔧 [EMBEDDED] Setting up Android environment...");
        android_config.setup_android_environment();
        println!("✅ [EMBEDDED] Android environment configured");
        info!("✅ [EMBEDDED] Android environment configured");
        
        println!("⚡ [EMBEDDED] Creating Tokio runtime...");
        eprintln!("⚡ [EMBEDDED] Creating Tokio runtime...");
        info!("⚡ [EMBEDDED] Creating Tokio runtime...");
        let rt = match std::panic::catch_unwind(|| {
            tokio::runtime::Runtime::new()
        }) {
            Ok(Ok(runtime)) => {
                println!("✅ [EMBEDDED] Tokio runtime created successfully");
                info!("✅ [EMBEDDED] Tokio runtime created successfully");
                Arc::new(Mutex::new(runtime))
            },
            Ok(Err(e)) => {
                println!("❌ [EMBEDDED] Failed to create Tokio runtime: {}", e);
                eprintln!("❌ [EMBEDDED] Failed to create Tokio runtime: {}", e);
                error!("❌ [EMBEDDED] Failed to create Tokio runtime: {}", e);
                return Err(format!("Failed to create Tokio runtime: {}", e).into());
            },
            Err(_) => {
                println!("❌ [EMBEDDED] PANIC in Tokio runtime creation");
                eprintln!("❌ [EMBEDDED] PANIC in Tokio runtime creation");
                error!("❌ [EMBEDDED] PANIC in Tokio runtime creation");
                return Err("Panic in Tokio runtime creation".into());
            }
        };
        
        let rt_clone = rt.clone();
        
        // Canal pour recevoir le statut du serveur
        let (tx, rx) = mpsc::channel::<Result<String, String>>();
        
        println!("🧵 [EMBEDDED] Starting server thread...");
        eprintln!("🧵 [EMBEDDED] Starting server thread...");
        info!("🧵 [EMBEDDED] Starting server thread...");
        let tx_clone = tx.clone();
        let _server_handle = thread::spawn(move || {
            println!("📡 Server thread started - THREAD RUNNING");
            eprintln!("📡 Server thread started - THREAD RUNNING");
            info!("📡 Server thread started");
            let rt_guard = match rt_clone.lock() {
                Ok(guard) => {
                    println!("✅ Runtime lock acquired successfully");
                    guard
                },
                Err(e) => {
                    println!("❌ Failed to acquire runtime lock: {}", e);
                    eprintln!("❌ Failed to acquire runtime lock: {}", e);
                    error!("❌ Failed to acquire runtime lock: {}", e);
                    let _ = tx_clone.send(Err(format!("Runtime lock failed: {}", e)));
                    return;
                }
            };
            
            // Launch the server in the background
            let _server_task = rt_guard.spawn(async move {
                println!("🔧 Starting full web server - ASYNC TASK STARTED");
                eprintln!("🔧 Starting full web server - ASYNC TASK STARTED");
                info!("🔧 Starting full web server...");
                
                println!("🔧 Environment variables check:");
                println!("   - DATABASE_URL: {}", std::env::var("DATABASE_URL").unwrap_or("NOT_SET".to_string()));
                println!("   - SERVER_HOST: {}", std::env::var("SERVER_HOST").unwrap_or("NOT_SET".to_string()));
                println!("   - SERVER_PORT: {}", std::env::var("SERVER_PORT").unwrap_or("NOT_SET".to_string()));
                println!("   - ENVIRONMENT: {}", std::env::var("ENVIRONMENT").unwrap_or("NOT_SET".to_string()));
                
                info!("🔧 Environment variables check:");
                info!("   - DATABASE_URL: {}", std::env::var("DATABASE_URL").unwrap_or("NOT_SET".to_string()));
                info!("   - SERVER_HOST: {}", std::env::var("SERVER_HOST").unwrap_or("NOT_SET".to_string()));
                info!("   - SERVER_PORT: {}", std::env::var("SERVER_PORT").unwrap_or("NOT_SET".to_string()));
                info!("   - ENVIRONMENT: {}", std::env::var("ENVIRONMENT").unwrap_or("NOT_SET".to_string()));
                
                println!("🔧 About to call start_full_web_server() - CRITICAL CALL");
                eprintln!("🔧 About to call start_full_web_server() - CRITICAL CALL");
                info!("🔧 About to call start_full_web_server()...");
                
                // Essayer d'abord le serveur complet
                let server_future = start_full_web_server();
                let timeout_duration = tokio::time::Duration::from_secs(10);
                
                println!("🔧 Calling start_full_web_server with 10 second timeout...");
                eprintln!("🔧 Calling start_full_web_server with 10 second timeout...");
                
                match tokio::time::timeout(timeout_duration, server_future).await {
                    Ok(Ok(_)) => {
                        println!("✅ Full web server started successfully - SHOULD NEVER HAPPEN");
                        eprintln!("✅ Full web server started successfully - SHOULD NEVER HAPPEN");
                        info!("✅ Full web server started successfully");
                        let _ = tx_clone.send(Ok("Full server started".to_string()));
                    },
                    Ok(Err(e)) => {
                        println!("⚠️ Full web server failed: {} - TRYING FALLBACK", e);
                        eprintln!("⚠️ Full web server failed: {} - TRYING FALLBACK", e);
                        warn!("⚠️ Full web server failed: {} - trying fallback", e);
                        
                        // Essayer le serveur de fallback
                        match start_simple_fallback_server().await {
                            Ok(_) => {
                                println!("✅ Fallback server started successfully");
                                eprintln!("✅ Fallback server started successfully");
                                info!("✅ Fallback server started successfully");
                                let _ = tx_clone.send(Ok("Fallback server started".to_string()));
                            },
                            Err(fallback_err) => {
                                println!("❌ Both servers failed: main={}, fallback={}", e, fallback_err);
                                eprintln!("❌ Both servers failed: main={}, fallback={}", e, fallback_err);
                                error!("❌ Both servers failed: main={}, fallback={}", e, fallback_err);
                                let _ = tx_clone.send(Err(format!("Both servers failed: main={}, fallback={}", e, fallback_err)));
                            }
                        }
                    },
                    Ok(_) => {
                        // This arm covers any other Ok(_) variant not matched above
                        println!("⚠️ Full web server returned unexpected Ok variant - TRYING FALLBACK");
                        eprintln!("⚠️ Full web server returned unexpected Ok variant - TRYING FALLBACK");
                        warn!("⚠️ Full web server returned unexpected Ok variant - trying fallback");
                        match start_simple_fallback_server().await {
                            Ok(_) => {
                                println!("✅ Fallback server started successfully (unexpected Ok variant)");
                                eprintln!("✅ Fallback server started successfully (unexpected Ok variant)");
                                info!("✅ Fallback server started successfully (unexpected Ok variant)");
                                let _ = tx_clone.send(Ok("Fallback server started (unexpected Ok variant)".to_string()));
                            },
                            Err(fallback_err) => {
                                println!("❌ Both servers failed (unexpected Ok variant): fallback={}", fallback_err);
                                eprintln!("❌ Both servers failed (unexpected Ok variant): fallback={}", fallback_err);
                                error!("❌ Both servers failed (unexpected Ok variant): fallback={}", fallback_err);
                                let _ = tx_clone.send(Err(format!("Both servers failed (unexpected Ok variant): fallback={}", fallback_err)));
                            }
                        }
                    },
                    Err(_timeout_err) => {
                        println!("⏰ Full web server timeout - TRYING FALLBACK");
                        eprintln!("⏰ Full web server timeout - TRYING FALLBACK");
                        warn!("⏰ Full web server timeout - trying fallback");
                        
                        // Essayer le serveur de fallback
                        match start_simple_fallback_server().await {
                            Ok(_) => {
                                println!("✅ Fallback server started successfully after timeout");
                                eprintln!("✅ Fallback server started successfully after timeout");
                                info!("✅ Fallback server started successfully after timeout");
                                let _ = tx_clone.send(Ok("Fallback server started after timeout".to_string()));
                            },
                            Err(fallback_err) => {
                                println!("❌ Fallback server also failed: {}", fallback_err);
                                eprintln!("❌ Fallback server also failed: {}", fallback_err);
                                error!("❌ Fallback server also failed: {}", fallback_err);
                                let _ = tx_clone.send(Err(format!("Both servers failed: timeout and fallback={}", fallback_err)));
                            }
                        }
                    }
                }
                
                // Give server time to initialize
                println!("🔧 Giving server time to initialize (5 seconds)...");
                eprintln!("🔧 Giving server time to initialize (5 seconds)...");
                info!("🔧 Giving server time to initialize (5 seconds)...");
                
                // Give the server substantial time to start up
                tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
                
                println!("🔧 Testing server connection...");
                eprintln!("🔧 Testing server connection...");
                info!("🔧 Testing server connection...");
                let port = std::env::var("SERVER_PORT").unwrap_or("8088".to_string());
                
                // Try multiple connection approaches with more aggressive testing
                let test_urls = vec![
                    format!("http://127.0.0.1:{}/api/ping", port),
                    format!("http://localhost:{}/api/ping", port),
                ];
                
                let mut connection_successful = false;
                
                // First try simple TCP connection to verify the port is open
                info!("🔗 First, testing raw TCP connection...");
                for attempt in 1..=10 {
                    match tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await {
                        Ok(_) => {
                            info!("✅ TCP connection successful on attempt {}!", attempt);
                            break;
                        },
                        Err(e) => {
                            if attempt < 10 {
                                warn!("⚠️ TCP connection attempt {} failed: {}, retrying...", attempt, e);
                                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                            } else {
                                error!("❌ All TCP connection attempts failed: {}", e);
                                let _ = tx_clone.send(Err(format!("TCP connection failed: {}", e)));
                                return;
                            }
                        }
                    }
                }
                
                // Now test HTTP endpoints
                for (attempt, url) in test_urls.iter().enumerate() {
                    info!("🔗 HTTP Attempt {} - Testing URL: {}", attempt + 1, url);
                    
                    // Try multiple times for each URL
                    for retry in 1..=3 {
                        info!("   HTTP Retry {}/3 for {}", retry, url);
                        
                        match reqwest::Client::new()
                            .get(url)
                            .timeout(tokio::time::Duration::from_secs(3))
                            .send()
                            .await 
                        {
                            Ok(response) => {
                                info!("✅ HTTP Response received! Status: {}", response.status());
                                if response.status().is_success() {
                                    info!("✅ Server is responding successfully at {}!", url);
                                    connection_successful = true;
                                    break;
                                } else {
                                    warn!("⚠️ Server responded but with status: {}", response.status());
                                }
                            },
                            Err(e) => {
                                warn!("⚠️ HTTP Connection attempt failed: {}", e);
                                // Wait before retry
                                if retry < 3 {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                                }
                            }
                        }
                    }
                    
                    if connection_successful {
                        break;
                    }
                }
                
                if connection_successful {
                    let _ = tx_clone.send(Ok("Server started and responding to HTTP".to_string()));
                } else {
                    // If HTTP fails but TCP works, server is listening but may not have all endpoints ready
                    warn!("⚠️ HTTP endpoints not responding, but TCP connection works - server may be partially ready");
                    let _ = tx_clone.send(Ok("Server started (TCP verified, HTTP pending)".to_string()));
                }
            });
        });
        
        SERVER_RUNTIME = Some(rt);
        
        info!("⏱️ Waiting for server startup...");
        info!("🔧 [DEBUG] About to wait for server with 15 second timeout...");
        let start_time = std::time::Instant::now();
        
        match rx.recv_timeout(Duration::from_millis(15000)) { // Augmenté à 15 secondes
            Ok(Ok(message)) => {
                let elapsed = start_time.elapsed();
                info!("✅ Server startup success: {} (took {:?})", message, elapsed);
                
                // Double-vérification finale avec TCP depuis le thread principal
                let port = std::env::var("SERVER_PORT").unwrap_or("8088".to_string()).parse::<u16>().unwrap_or(8088);
                info!("🔧 [DEBUG] Final verification: testing TCP connection to 127.0.0.1:{}...", port);
                match TcpStream::connect(("127.0.0.1", port)) {
                    Ok(_) => {
                        info!("✅ Final verification: TCP connection successful from main thread");
                        Ok(())
                    },
                    Err(e) => {
                        error!("❌ Final verification failed: {}", e);
                        Err(format!("Final verification failed: {}", e).into())
                    }
                }
            },
            Ok(Err(error)) => {
                let elapsed = start_time.elapsed();
                error!("❌ Server startup failed: {} (after {:?})", error, elapsed);
                Err(format!("Server startup failed: {}", error).into())
            },
            Err(_) => {
                let elapsed = start_time.elapsed();
                error!("⏰ Server startup timeout after {:?} (expected 15 seconds max)", elapsed);
                
                // Diagnostic final en cas de timeout
                let port = std::env::var("SERVER_PORT").unwrap_or("8088".to_string()).parse::<u16>().unwrap_or(8088);
                error!("🔧 [TIMEOUT DEBUG] Testing if server is listening malgré le timeout...");
                match TcpStream::connect(("127.0.0.1", port)) {
                    Ok(_) => {
                        error!("🤔 [TIMEOUT DEBUG] Le serveur EST à l'écoute mais le thread de vérification a échoué !");
                        Ok(()) // Le serveur fonctionne en réalité
                    },
                    Err(e) => {
                        error!("❌ [TIMEOUT DEBUG] Le serveur n'est PAS à l'écoute : {}", e);
                        Err("Server startup timeout after 15 seconds".into())
                    }
                }
            }
        }
    }
}

/// Get the server URL 
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_getServerUrl(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    init_logging();
    
    let port = std::env::var("SERVER_PORT").unwrap_or("8088".to_string());
    // Always return 127.0.0.1 for client connections, regardless of server bind address
    let server_url = format!("http://127.0.0.1:{}", port);
    
    info!("🌐 Providing server URL: {}", server_url);
    
    let output = env.new_string(server_url)
        .expect("Couldn't create java string!");
    
    output.into_raw()
}

/// Test server connectivity
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_testServerConnectivity(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    init_logging();
    
    let host = std::env::var("SERVER_HOST").unwrap_or("127.0.0.1".to_string());
    let port = std::env::var("SERVER_PORT").unwrap_or("8088".to_string()).parse::<u16>().unwrap_or(8088);
    
    info!("🔍 Testing connectivity to {}:{}", host, port);
    
    match TcpStream::connect((host.clone(), port)) {
        Ok(_) => {
            info!("✅ Successfully connected to server at {}:{}", host, port);
            1 // true
        },
        Err(e) => {
            info!("❌ Failed to connect to server at {}:{}: {}", host, port, e);
            0 // false
        }
    }
}

/// Test function to verify JNI binding works
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_testJNI(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    // Test ultra simple - juste retourner 1 (true)
    1
}

/// Debug server status - helper function to diagnose server issues
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_debugServerStatus(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    init_logging();
    
    info!("🔍 [DEBUG] Starting server diagnostics...");
    
    // Check environment variables
    info!("🔍 [DEBUG] Environment variables:");
    info!("   - DATABASE_URL: {}", std::env::var("DATABASE_URL").unwrap_or("NOT_SET".to_string()));
    info!("   - SERVER_HOST: {}", std::env::var("SERVER_HOST").unwrap_or("NOT_SET".to_string()));
    info!("   - SERVER_PORT: {}", std::env::var("SERVER_PORT").unwrap_or("NOT_SET".to_string()));
    info!("   - ENVIRONMENT: {}", std::env::var("ENVIRONMENT").unwrap_or("NOT_SET".to_string()));
    info!("   - SSL_ENABLED: {}", std::env::var("SSL_ENABLED").unwrap_or("NOT_SET".to_string()));
    info!("   - CORS_PERMISSIVE: {}", std::env::var("CORS_PERMISSIVE").unwrap_or("NOT_SET".to_string()));
    
    // Check server runtime status
    unsafe {
        match &SERVER_RUNTIME {
            Some(rt) => {
                info!("✅ [DEBUG] Server runtime exists");
                match rt.try_lock() {
                    Ok(_) => info!("✅ [DEBUG] Server runtime is accessible"),
                    Err(e) => info!("⚠️ [DEBUG] Server runtime is locked: {}", e),
                }
            },
            None => {
                info!("❌ [DEBUG] No server runtime found");
                return 0;
            }
        }
    }
    
    // Test port availability
    let port = std::env::var("SERVER_PORT").unwrap_or("8088".to_string()).parse::<u16>().unwrap_or(8088);
    info!("🔍 [DEBUG] Testing port {} availability...", port);
    
    match TcpStream::connect(("127.0.0.1", port)) {
        Ok(_) => {
            info!("✅ [DEBUG] Port {} is open and accepting connections", port);
            1
        },
        Err(e) => {
            info!("❌ [DEBUG] Port {} is not accessible: {}", port, e);
            0
        }
    }
}

/// Test version simplifiée pour debug - ne lance pas le vrai serveur
#[no_mangle]
pub extern "C" fn Java_com_main_MainActivity_initRustDebug(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    println!("=== DEBUG RUST FUNCTION CALLED ===");
    eprintln!("=== DEBUG RUST FUNCTION CALLED (stderr) ===");
    
    // Initialiser le logging en premier
    init_logging();
    
    info!("🔧 [DEBUG] initRustDebug() called from Java");
    println!("🔧 [DEBUG] initRustDebug() called from Java - println");
    eprintln!("🔧 [DEBUG] initRustDebug() called from Java - eprintln");
    
    // Test des variables d'environnement Android
    let android_config = AndroidServerConfig::new();
    println!("🔧 [DEBUG] AndroidServerConfig created");
    android_config.setup_android_environment();
    println!("🔧 [DEBUG] Android environment configured");
    
    // Vérifier les variables
    println!("🔧 [DEBUG] DATABASE_URL: {}", std::env::var("DATABASE_URL").unwrap_or("NOT_SET".to_string()));
    println!("🔧 [DEBUG] SERVER_HOST: {}", std::env::var("SERVER_HOST").unwrap_or("NOT_SET".to_string()));
    println!("🔧 [DEBUG] SERVER_PORT: {}", std::env::var("SERVER_PORT").unwrap_or("NOT_SET".to_string()));
    println!("🔧 [DEBUG] ENVIRONMENT: {}", std::env::var("ENVIRONMENT").unwrap_or("NOT_SET".to_string()));
    
    // Simuler un délai et retourner succès
    std::thread::sleep(std::time::Duration::from_millis(2000));
    
    println!("✅ [DEBUG] Debug initialization completed successfully");
    eprintln!("✅ [DEBUG] Debug initialization completed successfully");
    1 // true
}

/// Simple fallback server that just provides basic functionality
async fn start_simple_fallback_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("🔧 [FALLBACK] Starting simple fallback server...");
    
    // Pour l'instant, on simule un serveur simple qui fonctionne
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8088".to_string()).parse::<u16>().unwrap_or(8088);
    
    info!("🔧 [FALLBACK] Simulating server on {}:{}", host, port);
    
    // Simuler un délai pour le démarrage
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    info!("🔧 [FALLBACK] Fallback server simulation completed");
    
    // Pour l'instant, on retourne une erreur pour utiliser le serveur principal
    Err("Fallback server not fully implemented yet".into())
}
