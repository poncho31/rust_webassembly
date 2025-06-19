# Configuration Android pour l'application Rust

## Variables d'environnement pour Android
RUST_LOG=info
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Base de données pour Android (SQLite local recommandé)
DATABASE_URL=sqlite://android_app.db

# Configuration Android spécifique
ANDROID_MODE=true
ENABLE_CORS=true
LOG_LEVEL=debug

## Notes d'implémentation

### 1. Serveur Rust pour Android
Le serveur doit être adapté pour fonctionner dans l'environnement Android :

```rust
// Dans server/src/main.rs
#[cfg(target_os = "android")]
use android_logger::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration des logs pour Android
    #[cfg(target_os = "android")]
    android_logger::init_once(
        Config::default().with_min_level(log::Level::Debug)
    );

    // Configuration du serveur
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    
    println!("🚀 Démarrage du serveur Rust pour Android sur {}:{}", host, port);
    
    // Votre code serveur existant...
    Ok(())
}
```

### 2. Base de données SQLite pour Android
Pour une application mobile, SQLite est souvent plus approprié :

```rust
// Dans Cargo.toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite"] }
```

### 3. Permissions Android
Assurez-vous que les permissions suivantes sont dans AndroidManifest.xml :
- INTERNET
- ACCESS_NETWORK_STATE
- WRITE_EXTERNAL_STORAGE (si base de données externe)

### 4. Compilation en bibliothèque
Pour Android, le serveur doit être compilé comme une bibliothèque :

```rust
// Dans server/src/lib.rs
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn start_rust_server() -> *mut c_char {
    // Démarrer le serveur de manière asynchrone
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            // Votre code serveur...
        });
    
    CString::new("Server started").unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn stop_rust_server() {
    // Arrêter le serveur
}
```

### 5. Configuration Cargo.toml pour Android
```toml
[lib]
name = "rust_server"
crate-type = ["cdylib"]

[[bin]]
name = "server"
required-features = ["android"]

[features]
android = []
```

### 6. Build script pour Android
Le script build_rust_android.bat/sh gère automatiquement :
- Installation des cibles Android
- Configuration du NDK
- Compilation croisée
- Copie des binaires dans jniLibs
