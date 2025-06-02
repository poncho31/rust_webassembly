# 🏗️ WebAssembly Unified Frontend-Backend Application
## Définitions Techniques et Fonctionnelles

---

## 📋 DESCRIPTION GÉNÉRALE

### 🎯 **Objectif de l'Application**
Application web moderne démontrant l'intégration complète entre **Rust WebAssembly** (frontend) et **Rust Actix-Web** (backend) pour créer une Single Page Application (SPA) unifiée avec fonctionnalités de formulaire, validation, stockage de fichiers et rafraîchissement automatique.

### 🌟 **Vision**
Créer un écosystème de développement web entièrement en Rust, éliminant les frontières traditionnelles entre frontend et backend tout en offrant des performances natives et une sécurité de type renforcée.

---

## 🏛️ ARCHITECTURE TECHNIQUE

### 📐 **Architecture Générale**
```
┌─────────────────────────────────────────────────────────────┐
│                    🌐 Browser (Client)                     │
├─────────────────────────────────────────────────────────────┤
│  🎨 HTML/CSS/JS  ←→  ⚡ WebAssembly (Rust)                │
├─────────────────────────────────────────────────────────────┤
│                    📡 HTTP/REST API                        │
├─────────────────────────────────────────────────────────────┤
│              🚀 Actix-Web Server (Rust)                    │
├─────────────────────────────────────────────────────────────┤
│               🗄️ File System Storage                       │
└─────────────────────────────────────────────────────────────┘
```

### 🧩 **Modules et Structure**

#### **Workspace Cargo Structure**
```rust
[workspace]
resolver = "2"
members = [
    "server",    // Backend Actix-Web
    "client",    // Frontend WebAssembly
    "core"       // Shared utilities
]
```

---

## 💻 COMPOSANTS TECHNIQUES

### ⚡ **Frontend - WebAssembly Client**

#### **Technologies Principales**
- **Language**: Rust 2021 Edition
- **Framework**: wasm-bindgen + web-sys
- **Target**: `wasm32-unknown-unknown`
- **Compilation**: wasm-pack (target web)

#### **Dépendances Clés**
```toml
wasm-bindgen = "0.2"           # Interface JS-WASM
web-sys = "0.3.77"             # API Web natives
serde = "1.0"                  # Sérialisation
gloo-timers = "0.3"            # Timers asynchrones
futures = "0.3"                # Programmation async
```

#### **Modules Frontend**
```rust
client/src/
├── lib.rs                     # Point d'entrée principal
├── client_tools.rs            # Utilitaires logging/debug
├── client_request.rs          # Client HTTP WASM
├── client_periodics.rs        # Tâches périodiques
├── modal.rs                   # Système de modales
├── form/                      # Système de formulaires
│   ├── handler.rs            # Gestionnaire principal
│   ├── config.rs             # Configuration
│   ├── field.rs              # Types de champs
│   ├── processor.rs          # Traitement données
│   ├── errors.rs             # Gestion erreurs
│   └── cache.rs              # Cache performance
├── validation/                # Système validation
│   ├── improved.rs           # Validateurs avancés
│   └── tests.rs              # Tests unitaires
└── refresh/                   # Rafraîchissement auto
    ├── config.rs             # Configuration refresh
    ├── handler.rs            # Gestionnaire refresh
    └── scheduler.rs          # Planificateur
```

### 🚀 **Backend - Actix-Web Server**

#### **Technologies Principales**
- **Framework**: Actix-Web 4.x
- **Runtime**: Tokio (async)
- **Serialization**: Serde JSON
- **CORS**: Actix-CORS
- **File handling**: Actix-Multipart

#### **Dépendances Clés**
```toml
actix-web = "4"               # Framework web
actix-files = "0.6"           # Serveur fichiers statiques
actix-cors = "0.6"            # Gestion CORS
tokio = "1"                   # Runtime asynchrone
serde = "1.0"                 # Sérialisation
sqlx = "0.7"                  # ORM base de données
```

#### **Modules Backend**
```rust
server/src/
├── main.rs                   # Point d'entrée serveur
├── extract_form.rs           # Extraction données multipart
├── controllers/              # Contrôleurs API
│   ├── form_controller.rs    # API formulaires
│   ├── ping_controller.rs    # API ping/health
│   └── weather_controller.rs # API météo
└── models/                   # Modèles de données
    └── form_response.rs      # Modèles réponse
```

### 🔧 **Core Library - Utilitaires Partagés**

#### **Modules Core**
```rust
core/src/
├── lib.rs                    # Exports principaux
├── database.rs               # Utilitaires DB
├── config/                   # Configuration partagée
├── db_models/                # Modèles base de données
│   └── user.rs              # Modèle utilisateur
├── errors/                   # Types d'erreurs
└── http_models/              # Modèles HTTP
    └── http_responses.rs     # Réponses standardisées
```

---

## 🔧 FONCTIONNALITÉS

### 📝 **Système de Formulaires Avancé**

#### **Caractéristiques**
- **Validation temps réel** côté client et serveur
- **Types de champs supportés**: Text, Email, Date, Number, Select, File, TextArea
- **Upload de fichiers multiples** avec limitation de taille (10MB)
- **Retry automatique** en cas d'échec (3 tentatives)
- **Messages de feedback** personnalisables
- **Configuration dynamique** des champs

#### **Configuration des Champs**
```rust
// Exemple de configuration de champ Select
let sexe_options = vec![
    FieldOption::new("", "Sélectionnez..."),
    FieldOption::new("homme", "Homme"),
    FieldOption::new("femme", "Femme"),
    FieldOption::new("autre", "Autre"),
];
field_configs.insert("sexe", FieldConfig::new(FieldType::Select)
    .with_options(sexe_options)
    .required());
```

#### **Validation Avancée**
```rust
let validator = FormValidator::new()
    .add_rule("login", ValidationRule::text(3, 20))
    .add_rule("email", ValidationRule::email())
    .add_rule("age", ValidationRule::number(0.0, 150.0));
```

### 🔄 **Système de Rafraîchissement Automatique**

#### **Caractéristiques**
- **Mise à jour périodique** configurable (10s, 30s, 60s)
- **Transformation des données** (préfixes, suffixes, formats)
- **Gestion d'erreurs** avec affichage utilisateur
- **Support paramètres dynamiques** via champs input

#### **Types de Rafraîchissement**
1. **Texte simple** - Mise à jour textContent
2. **HTML** - Mise à jour innerHTML  
3. **Valeur input** - Mise à jour de champs
4. **Attributs** - Modification d'attributs DOM

#### **Configuration Exemple**
```rust
let temperature_config = RefreshConfig::new_text(
    "temperature",
    "/api/weather/temperature", 
    30,  // Toutes les 30 secondes
    "#auto-server-status",
    Some("temperature"),
).with_transform(DataTransform {
    prefix: Some("🌡️ ".to_string()),
    suffix: Some("°C".to_string()),
    format: Some("number".to_string()),
}).with_input_field("#region"); // Utilise champ région
```

### 🌡️ **API Météo Intelligente**

#### **Endpoint**: `GET /api/weather/temperature`
#### **Paramètres**: 
- `region` (optionnel) - Nom de la ville (défaut: "Bruxelles")

#### **Villes Supportées**
- **Belgique**: Bruxelles, Anvers, Gand, Liège, Charleroi
- **Europe**: Paris, London, Berlin, Amsterdam, Madrid, Rome
- **Monde**: New York, Tokyo, Sydney, Montreal

#### **Réponse JSON**
```json
{
  "temperature": 18.5,
  "region": "Bruxelles",
  "status": 200,
  "message": "Température actuelle pour Bruxelles"
}
```

### 📡 **APIs REST Disponibles**

#### **1. POST/GET /api/ping**
- **Fonction**: Health check du serveur
- **Réponse**: Status HTTP et message

#### **2. POST /api/form**
- **Fonction**: Soumission de formulaire avec fichiers
- **Support**: Multipart form data
- **Validation**: Côté serveur complète
- **Stockage**: Fichiers dans `/storage/files/`

#### **3. GET /api/weather/temperature**
- **Fonction**: Récupération température par région
- **Paramètres**: `?region=ville`
- **Données**: Simulées avec variabilité

---

## 🎨 INTERFACE UTILISATEUR

### 🌐 **Technologie Frontend**
- **HTML5** sémantique avec support accessibility
- **CSS3** avec Grid/Flexbox et animations
- **JavaScript ES6+** modules pour internationalisation
- **WebAssembly** pour logique métier

### 🎨 **Design Système**

#### **Layout Responsive**
- **Grid CSS** pour formulaires adaptatifs
- **Cartes modulaires** pour sections rafraîchissement
- **Animations CSS** pour feedback utilisateur

#### **Composants UI**
- **Formulaires adaptatifs** avec validation visuelle
- **Modales** pour feedback succès/erreur
- **Indicateurs de chargement** avec animations
- **Sélecteurs de fichiers** avec preview
- **Cartes temps réel** pour données live

### 🌍 **Internationalisation**
- **Support Français/Anglais**
- **Traductions dynamiques** via `translations.js`
- **Labels contextuels** avec `data-translate`

---

## 📊 PERFORMANCES ET OPTIMISATION

### ⚡ **Optimisations WebAssembly**
- **Compilation optimisée** avec wasm-opt `-O4`
- **Taille minimale** via tree-shaking
- **Chargement asynchrone** avec lazy loading

### 🚀 **Optimisations Serveur**
- **Worker unique** pour développement
- **CORS optimisé** avec cache headers
- **Serveur de fichiers statiques** intégré
- **Logging structuré** avec env_logger

### 💾 **Gestion Mémoire**
- **Cache intelligent** côté client
- **Monitoring performance** intégré
- **Cleanup automatique** des ressources

---

## 🔒 SÉCURITÉ

### 🛡️ **Validation Multi-Niveaux**
1. **Client-side** (WebAssembly) - UX immédiate
2. **Server-side** (Rust) - Sécurité garantie
3. **Type safety** - Rust compile-time checks

### 🔐 **Gestion Fichiers**
- **Limitation taille** (10MB max)
- **Validation types MIME**
- **Stockage sécurisé** dans dossier dédié
- **Noms de fichiers** UUID pour éviter conflits

### 🌐 **CORS et Headers**
- **CORS permissif** pour développement
- **Headers sécurisés** avec timeout
- **Validation origine** configurable

---

## 🚀 DÉPLOIEMENT

### 📦 **Build Process**
```bash
# Compilation complète
cmd /c "cd client & wasm-pack build --target web --out-dir static/pkg & cd .. & cargo run"
```

### 🔧 **Configuration Environnement**
```env
SERVER_HOST=127.0.0.1
SERVER_PORT=8088
HTML_INDEX=index.html
```

### 📁 **Structure de Déploiement**
```
static/
├── index.html              # Page principale
├── app.css                 # Styles globaux
├── translations.js         # I18n
├── pkg/                    # WebAssembly build
│   ├── client.js          # Interface JS
│   └── client_bg.wasm     # Module WASM
└── css/                    # Styles modulaires
    ├── modal.css          # Styles modales
    └── table.css          # Styles tableaux
```

---

## 📈 MONITORING ET MÉTRIQUES

### 📊 **Métriques Collectées**
- **Latence requêtes** API
- **Taux d'erreur** validation
- **Performance WASM** (temps chargement)
- **Utilisation mémoire** client/serveur

### 🔍 **Observabilité**
- **Logs structurés** avec timestamps
- **Console debugging** côté client
- **Health checks** automatiques
- **Performance timing** WebAssembly

---

## 🔮 ÉVOLUTIONS FUTURES

### 🎯 **Roadmap Technique**
1. **Base de données PostgreSQL** (SQLx intégré)
2. **Authentication/Authorization** JWT
3. **Tests automatisés** côté client et serveur
4. **CI/CD Pipeline** avec GitHub Actions
5. **PWA capabilities** (Service Workers)
6. **WebRTC** pour temps réel
7. **GraphQL** API alternative

### 🌟 **Améliorations UX**
1. **Offline support** avec caching
2. **Dark mode** toggle
3. **Drag & drop** amélioré
4. **Notifications push**
5. **Animations avancées**

---

## 📚 DOCUMENTATION TECHNIQUE

### 🔗 **Ressources**
- **Repository**: `webassembly_unified_frontbackend`
- **License**: MIT
- **Rust Edition**: 2021
- **WASM Target**: `wasm32-unknown-unknown`

### 🛠️ **Outils de Développement**
- **wasm-pack**: Build et packaging WebAssembly
- **cargo-watch**: Hot reload développement
- **wasm-bindgen**: Interface JS-WASM
- **web-sys**: API Web pour WebAssembly

---

*Document généré automatiquement - Version 1.0 - Juin 2025*
