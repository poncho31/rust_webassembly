# WebAssembly Unified - Android

## Installation Autonome

Ce script installe **TOUT localement** dans le dossier `platforms/android/` :
- ✅ **Java JDK 17** (local)
- ✅ **Android SDK** (local)
- ✅ **cargo-ndk** (local)
- ✅ **Gradle Wrapper** (local)
- ✅ **Targets Rust Android** (gérés automatiquement)

**Aucune installation système requise !**

## Prérequis

**SEULEMENT** :
- **Rust** installé (rustc + cargo)

## Construction de l'APK

### Méthode simple (recommandée)
```bash
# Depuis la racine du projet
_run.bat android
```

### Méthode directe
```bash
cd platforms/android
build_android.bat
```

### Nettoyage complet
```bash
cd platforms/android
clean_android.bat
```

## Architecture

- **MainActivity.kt** : Interface Android utilisant une WebView
- **lib.rs** : Code Rust compilé en bibliothèque native (.so)
- **WebView** : Affiche le contenu de votre serveur web Rust
- **JavaScript Interface** : Communication bidirectionnelle entre Android et Rust

## Communication

L'application Android charge votre serveur web dans une WebView et peut :
- Afficher l'interface web existante
- Communiquer avec le backend Rust via JavaScript
- Traiter les messages côté Rust

## Fichiers générés

- `app/build/outputs/apk/debug/app-debug.apk` : APK de développement
- `app/src/main/jniLibs/` : Bibliothèques natives Rust pour chaque architecture

## Installation

L'APK sera automatiquement proposé pour installation sur un appareil connecté après la compilation.
