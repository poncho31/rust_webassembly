# 🚫 YouTube AdBlock Proxy - Guide d'Utilisation

## 📋 Description

Ce proxy Rust/WebAssembly bloque automatiquement les publicités YouTube en filtrant les requêtes et en injectant du CSS anti-publicité. Le système fonctionne en mode proxy HTTP transparent.

## 🚀 Démarrage

### Mode Proxy AdBlock (Port 8089)
```bash
cargo run -- --cmd "addblock"
```

### Mode Serveur Web Complet (Port 8088)
```bash
cargo run
```

## 🔧 Configuration

### Configuration Proxy dans le Navigateur

#### Chrome/Edge
1. Paramètres → Avancé → Système
2. Ouvrir les paramètres proxy de votre ordinateur
3. Configuration manuelle du proxy :
   - **HTTP Proxy:** `127.0.0.1:8089`
   - **HTTPS Proxy:** `127.0.0.1:8089`

#### Firefox
1. Paramètres → Général → Paramètres réseau
2. Configuration manuelle du proxy :
   - **Proxy HTTP:** `127.0.0.1` Port `8089`
   - **Proxy HTTPS:** `127.0.0.1` Port `8089`

### URL Directe (Alternative)
Au lieu de configurer le proxy système, utilisez l'URL directe :
```
http://127.0.0.1:8089/proxy/youtube/{chemin_youtube}
```

Exemple :
```
http://127.0.0.1:8089/proxy/youtube/watch?v=dQw4w9WgXcQ
```

## 🛡️ Fonctionnalités de Blocage

### Domaines Bloqués (11 au total)
- `googleads.g.doubleclick.net`
- `googlesyndication.com`  
- `youtube.com/pagead`
- `youtube.com/get_video_info`
- `s.youtube.com/api/stats`
- `youtube.com/api/stats`
- `youtube.com/ptracking`
- `youtube.com/youtubei/v1/log_event`
- `youtube.com/generate_204`
- `yt3.ggpht.com/ytts/`
- `i.ytimg.com/an_webp/`

### CSS Anti-Publicité Injecté
Le proxy injecte automatiquement du CSS pour masquer :
- Lecteur publicitaire (`#player-ads`, `.ytp-ad-module`)
- Overlays publicitaires (`.ytp-ad-overlay-container`)
- Publicités de la sidebar (`#watch-sidebar-ads`)
- Contenu sponsorisé (`.ytd-promoted-*`)
- Et bien plus...

### Nettoyage HTML
Suppression automatique des scripts :
- Scripts Google Analytics
- Scripts DoubleClick
- Scripts de tracking publicitaire
- iFrames publicitaires

## 📊 Monitoring et Logs

### Endpoints de Status
```
GET http://127.0.0.1:8089/proxy/adblock/status
```

Retourne :
```json
{
  "status": "active",
  "statistics": {
    "total_requests_processed": 42,
    "ads_blocked": 15,
    "html_cleanings_performed": 8,
    "block_rate_percentage": 35.7
  },
  "blocked_domains": 11,
  "blocked_domains_list": [...],
  "features": [...],
  "timestamp": "2025-06-04T17:17:17.123Z"
}
```

### Logs Terminal en Temps Réel

Le serveur affiche en temps réel :
- ✅ **Requêtes proxifiées** : `PROXYING: GET -> https://www.youtube.com/watch?v=...`
- 🚫 **Publicités bloquées** : `BLOCKED: https://googleads.g.doubleclick.net/... [Total: 15]`
- 🧹 **Nettoyages HTML** : `CLEANED HTML: Removed 2043 bytes of ad content [Total: 8]`
- 💉 **Injections CSS** : `INJECTING: AdBlock CSS into HTML <head>`
- 📡 **Réponses reçues** : `Response received: status 200`

### Interface de Test Web

Ouvrez `adblock_test_interface.html` dans votre navigateur pour :
- Voir les statistiques en temps réel
- Tester des URLs YouTube
- Monitorer les logs
- Visualiser les domaines bloqués

## 🧪 Tests

### Test Simple
```bash
curl "http://127.0.0.1:8089/proxy/youtube/"
```

### Test avec Publicité (doit être bloqué)
```bash
curl "http://127.0.0.1:8089/proxy/youtube/pagead/ads"
```

### Test Status
```bash
curl "http://127.0.0.1:8089/proxy/adblock/status"
```

## ⚙️ Architecture Technique

### Flux de Données
1. **Requête Client** → Proxy Rust (port 8089)
2. **Filtrage URL** → Vérification domaines bloqués
3. **Requête Upstream** → YouTube (si non bloqué)
4. **Traitement Réponse** → Nettoyage HTML + Injection CSS
5. **Réponse Client** → Contenu sans publicité

### Technologies Utilisées
- **Actix-Web** : Framework web asynchrone
- **AWC** : Client HTTP pour les requêtes upstream
- **Regex** : Nettoyage HTML des scripts publicitaires
- **Serde** : Sérialisation JSON pour les APIs
- **Chrono** : Timestamps pour les logs

### Structure du Code
```
server/src/controllers/proxy_controller.rs
├── is_blocked_url()        # Détection des domaines publicitaires
├── proxy_youtube()         # Logique principale du proxy
├── adblock_status()        # Endpoint de monitoring
└── configure_proxy_routes() # Configuration des routes
```

## 🔍 Dépannage

### Le serveur ne démarre pas
- Vérifiez que le port 8089 n'est pas utilisé : `netstat -an | findstr 8089`
- Arrêtez les processus existants : `taskkill /F /IM server.exe`

### Pas de blocage de publicité
- Vérifiez les logs du terminal pour les messages `BLOCKED:`
- Testez l'endpoint status : `curl http://127.0.0.1:8089/proxy/adblock/status`
- Vérifiez la configuration proxy du navigateur

### Erreurs de proxy
- Assurez-vous d'utiliser HTTP (pas HTTPS) pour le proxy
- Vérifiez que YouTube est accessible directement
- Consultez les logs d'erreur dans le terminal

## 📈 Métriques de Performance

Le proxy enregistre automatiquement :
- **Requêtes traitées** : Nombre total de requêtes proxifiées
- **Publicités bloquées** : Compteur des URLs publicitaires interceptées  
- **Nettoyages HTML** : Nombre de pages où du contenu publicitaire a été supprimé
- **Taux de blocage** : Pourcentage de requêtes bloquées vs total

## 🛠️ Développement

### Ajouter de nouveaux domaines à bloquer
Modifiez `BLOCKED_DOMAINS` dans `proxy_controller.rs` :
```rust
static BLOCKED_DOMAINS: &[&str] = &[
    "nouveaudomaine.com",
    // ... domaines existants
];
```

### Améliorer le CSS anti-publicité
Modifiez `AD_BLOCK_CSS` pour cibler de nouveaux éléments :
```css
.nouveau-element-publicitaire {
    display: none !important;
}
```

### Ajouter des règles de nettoyage HTML
Modifiez la regex dans `get_html_cleaner()` :
```rust
Regex::new(r#"(?i)<script[^>]*(?:nouveau-pattern)[^>]*>.*?</script>"#)
```

## 📝 Notes

- Le proxy fonctionne uniquement pour YouTube actuellement
- Les logs sont affichés en temps réel dans le terminal
- Les statistiques sont persistantes pendant la session
- Le CSS injecté est compatible avec la version actuelle de YouTube

---

**Auteur**: Proxy AdBlock Rust  
**Version**: 1.0.0  
**Date**: Juin 2025
