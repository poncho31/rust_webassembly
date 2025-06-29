# Android Integration - Rust Server

Ce module intègre le serveur Rust existant (`start_full_web_server`) dans l'application Android.

## Architecture

### Structure refactorisée

```
platforms/android/src/
├── lib.rs              # Interface JNI principale
├── android_config.rs   # Configuration spécifique Android
└── ...
```

### Fonctionnalités

1. **Serveur Embedded** : Le serveur Rust complet s'exécute directement dans l'application Android
2. **Configuration automatique** : Paramètres optimisés pour l'environnement Android
3. **Gestion du cycle de vie** : Démarrage/arrêt contrôlé depuis Java/Kotlin
4. **Interface JNI complète** : Toutes les fonctions nécessaires exposées

## Fonctions JNI disponibles

### Gestion du serveur
- `initRust()` : Initialise et démarre le serveur embedded
- `getServerUrl()` : Retourne l'URL du serveur local (http://127.0.0.1:8080)
- `isServerRunning()` : Vérifie si le serveur est en cours d'exécution
- `stopServer()` : Arrête le serveur embedded
- `getServerStatus()` : Retourne le statut détaillé du serveur

### Communication
- `handleWebViewMessage(String)` : Traite les messages du WebView
- Messages JSON supportés pour communication bidirectionnelle

## Configuration Android

### Variables d'environnement automatiques
```
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
SSL_ENABLED=false
ENVIRONMENT=android
CORS_PERMISSIVE=true
COMPRESSION_ENABLED=true
SECURITY_HEADERS=false
FILE_CACHING=true
REQUEST_LOGGING=true
```

### Chemins de fichiers
```
Base: /data/data/com.main/files/
├── assets/     # Fichiers statiques
├── database/   # Base de données SQLite
└── logs/       # Fichiers de log
```

## Utilisation côté Android

### Kotlin/Java
```kotlin
class MainActivity : AppCompatActivity() {
    companion object {
        init {
            System.loadLibrary("webassembly_android")
        }
    }
    
    external fun initRust(): Boolean
    external fun getServerUrl(): String
    external fun isServerRunning(): Boolean
    external fun stopServer(): Boolean
    external fun getServerStatus(): String
    external fun handleWebViewMessage(message: String): String
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Démarrer le serveur Rust
        if (initRust()) {
            val serverUrl = getServerUrl()
            // Charger l'URL dans le WebView
            webView.loadUrl(serverUrl)
        }
    }
    
    override fun onDestroy() {
        super.onDestroy()
        stopServer()
    }
}
```

### WebView JavaScript
```javascript
// Communication avec le serveur local
fetch('http://127.0.0.1:8080/api/ping')
    .then(response => response.json())
    .then(data => console.log(data));

// Ou via l'interface Android
function sendMessageToRust(message) {
    return Android.handleWebViewMessage(JSON.stringify(message));
}
```

## Compilation

### Dépendances ajoutées
Le `Cargo.toml` Android inclut maintenant toutes les dépendances du serveur :
- `actix-web`, `actix-files`, `actix-cors`
- `sqlx`, `tokio`, `serde`
- `server_lib` (référence au serveur principal)

### Build
```bash
# Build pour Android
cd platforms/android
cargo build --target aarch64-linux-android --release

# Ou utiliser les scripts existants
./build_android.sh
```

## Avantages de cette architecture

1. **Réutilisation du code** : Le serveur complet est réutilisé sans duplication
2. **Fonctionnalités complètes** : Toutes les API REST sont disponibles localement
3. **Performance** : Pas de latence réseau, exécution native
4. **Sécurité** : Serveur local, pas d'exposition externe
5. **Développement simplifié** : Même API que la version serveur standalone

## Notes techniques

- Le serveur s'exécute dans un thread séparé avec un runtime Tokio
- La gestion mémoire est optimisée pour Android
- Les logs utilisent le système de logging Android
- Compatible avec les émulateurs et appareils physiques
- Support des architectures ARM64, ARM32, x86_64

## Troubleshooting

### Problèmes courants
1. **Port déjà utilisé** : Le serveur vérifie automatiquement
2. **Permissions manquantes** : Vérifier les permissions Android
3. **Chemins de fichiers** : Les répertoires sont créés automatiquement
4. **Mémoire insuffisante** : Ajuster les paramètres JVM si nécessaire
