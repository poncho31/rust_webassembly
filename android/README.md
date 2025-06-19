# Application Android avec Serveur Rust Intégré

Cette application Android utilise votre serveur Rust comme backend local, créant une application quasi-native avec d'excellentes performances.

## 🏗️ Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Interface     │────▶│   Serveur Rust   │────▶│   PostgreSQL   │
│   Android       │     │   (Local)        │     │   (Local/Cloud) │
│   (Java/Kotlin) │     │   Port 8080      │     │                 │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

## 🚀 Installation et Configuration

### Prérequis

1. **Android Studio** installé avec :
   - Android SDK (API 24+)
   - Android NDK (version 25+)
   - Build Tools

2. **Rust** configuré pour Android :
   ```bash
   # Installation des cibles Android
   rustup target add aarch64-linux-android
   rustup target add armv7-linux-androideabi  
   rustup target add x86_64-linux-android
   ```

3. **Variables d'environnement** :
   - `ANDROID_HOME` : Chemin vers Android SDK
   - `ANDROID_NDK_ROOT` : Chemin vers Android NDK

### Configuration du NDK

Modifiez les scripts de compilation (`build_rust_android.bat/sh`) pour pointer vers votre installation NDK :

```bash
# Linux/macOS
export ANDROID_NDK_ROOT="$HOME/Android/Sdk/ndk/25.2.9519653"

# Windows
set ANDROID_NDK_ROOT=%USERPROFILE%\AppData\Local\Android\Sdk\ndk\25.2.9519653
```

## 🔧 Compilation

### 1. Compiler le serveur Rust pour Android

```bash
# Windows
cd android
.\build_rust_android.bat

# Linux/macOS  
cd android
chmod +x build_rust_android.sh
./build_rust_android.sh
```

### 2. Compiler l'application Android

```bash
cd android
.\gradlew assembleDebug
```

### 3. Installer sur l'appareil

```bash
.\gradlew installDebug
```

## 📱 Fonctionnalités

### Interface Android
- **Démarrage/Arrêt** du serveur Rust intégré
- **Monitoring** du statut du serveur
- **Test API** pour vérifier la connectivité
- **Interface Material Design** moderne

### Serveur Rust Intégré
- **Serveur HTTP** local sur le port 8080
- **API REST** complète
- **Base de données** PostgreSQL (locale ou cloud)
- **Migrations** automatiques
- **Gestion d'erreurs** robuste

## 🗂️ Structure du Projet

```
android/
├── app/
│   ├── src/main/
│   │   ├── java/com/rustwebassembly/app/
│   │   │   ├── MainActivity.java          # Interface principale
│   │   │   └── RustServerService.java     # Service serveur Rust
│   │   ├── res/
│   │   │   ├── layout/
│   │   │   │   └── activity_main.xml      # Layout principal
│   │   │   ├── values/
│   │   │   │   ├── strings.xml
│   │   │   │   ├── colors.xml
│   │   │   │   └── themes.xml
│   │   │   └── ...
│   │   ├── jniLibs/                       # Binaires Rust
│   │   │   ├── arm64-v8a/
│   │   │   ├── armeabi-v7a/
│   │   │   └── x86_64/
│   │   └── AndroidManifest.xml
│   └── build.gradle                       # Configuration app
├── build.gradle                           # Configuration projet
├── settings.gradle
├── build_rust_android.bat/.sh            # Scripts compilation Rust
└── README.md
```

## 🔗 Communication Android ↔ Rust

### API Endpoints Disponibles
- `GET /ping` - Test de connectivité
- `GET /api/users` - Liste des utilisateurs
- `POST /api/users` - Créer un utilisateur
- `GET /api/logs` - Logs du système

### Exemple d'appel API
```java
// Dans MainActivity.java
String url = "http://127.0.0.1:8080/ping";
Request request = new Request.Builder().url(url).build();
httpClient.newCall(request).enqueue(callback);
```

## 🎯 Avantages de cette Architecture

### ✅ Performances
- **Vitesse native** : Rust compilé en natif
- **Pas de latence réseau** : Serveur local
- **UI native Android** : Pas de WebView

### ✅ Sécurité
- **Pas d'exposition réseau** : Serveur local uniquement
- **Contrôle total** : Aucune dépendance externe
- **Données locales** : Base de données locale possible

### ✅ Développement
- **Code réutilisable** : Même backend pour web/mobile
- **Maintenance facile** : Une seule codebase backend
- **Déploiement simple** : Application autonome

### ✅ Flexibilité
- **Offline-first** : Fonctionne sans internet
- **Évolutif** : Peut basculer vers serveur distant
- **Multi-plateforme** : iOS possible avec la même approche

## 🐛 Débogage

### Logs Android
```bash
adb logcat | grep "RustServerService\|MainActivity"
```

### Logs Rust
Les logs du serveur Rust apparaissent dans les logs Android avec le tag `RustServerService`.

### Problèmes Courants

1. **NDK non trouvé** : Vérifiez `ANDROID_NDK_ROOT`
2. **Binaires manquants** : Relancez `build_rust_android.bat`
3. **Serveur ne démarre pas** : Vérifiez les permissions Android
4. **API inaccessible** : Vérifiez que le serveur tourne sur le bon port

## 🚀 Prochaines Étapes

1. **Interface WebView** : Intégrer le client web existant
2. **Synchronisation** : Base de données locale + cloud
3. **Push notifications** : Alertes depuis le serveur
4. **Version iOS** : Porter vers iOS avec la même approche

## 📝 Notes Techniques

- **Port par défaut** : 8080 (configurable)
- **Architectures supportées** : ARM64, ARM32, x86_64
- **Android minimum** : API 24 (Android 7.0)
- **Permissions requises** : Internet, stockage
