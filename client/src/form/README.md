# Système de Formulaires - Guide Complet

## Vue d'ensemble

Ce système de formulaires offre une solution simple, performante et maintenable pour les applications WebAssembly. Il centralise toute la logique des formulaires dans un seul endroit avec une API cohérente.

## Structure des fichiers

### 📁 Organisation du dossier `form/`

```
form/
├── mod.rs                  # Point d'entrée principal - exporte tous les modules
├── form_config.rs          # Configuration des formulaires (options, paramètres)
├── form_field.rs           # Définition des champs (types, propriétés)
├── form_handler.rs         # Gestionnaire principal des formulaires
├── form_core.rs            # Logique centrale (traitement, erreurs)
├── form_validation.rs      # Système de validation complet
└── README_FR.md           # Ce fichier de documentation
```

## Rôle de chaque fichier

### 🔧 `mod.rs` - Point d'entrée
**Rôle :** Exporte tous les modules et types principaux du système
- Centralise les imports pour une utilisation simple
- Définit l'API publique du module form
- Permet d'importer tout avec `use crate::form::*`

### ⚙️ `form_config.rs` - Configuration
**Rôle :** Gère toutes les options de configuration des formulaires
- Validation activée/désactivée
- Messages de succès et d'erreur personnalisés
- Taille maximale des fichiers
- Nombre de tentatives en cas d'échec
- Indicateurs de chargement

**Exemple :**
```rust
let config = FormConfig::builder()
    .validation(true)
    .success_message("Formulaire envoyé avec succès !")
    .max_file_size(5 * 1024 * 1024) // 5MB
    .retry_attempts(3)
    .build();
```

### 📝 `form_field.rs` - Champs de formulaire
**Rôle :** Définit les types de champs et leurs propriétés
- Types : Text, Email, Password, TextArea, Select, Checkbox, Radio, File
- Validation intégrée pour chaque type
- Gestion automatique des attributs HTML

**Types disponibles :**
- `FieldType::Text` - Champ texte simple
- `FieldType::Email` - Champ email avec validation
- `FieldType::Password` - Champ mot de passe
- `FieldType::TextArea` - Zone de texte multiligne
- `FieldType::Select` - Liste déroulante
- `FieldType::Checkbox` - Case à cocher
- `FieldType::Radio` - Bouton radio
- `FieldType::File` - Upload de fichier

### 🎯 `form_handler.rs` - Gestionnaire principal
**Rôle :** Orchestre toute la logique du formulaire
- Pattern Builder pour une configuration flexible
- Gestion des événements (soumission, validation)
- Intégration avec le système de validation
- Gestion des erreurs et des tentatives

**Utilisation :**
```rust
let handler = FormHandler::new("mon-form", "/api/contact", config)?
    .with_field_specs(&fields)
    .build()?;

handler.initialize()?;
```

### 🏗️ `form_core.rs` - Logique centrale
**Rôle :** Contient la logique métier principale
- `FormProcessor` : Traitement et sérialisation des données
- `FormError` : Système d'erreurs typées
- Gestion des fichiers avec validation de taille
- Utilitaires de traitement des données

### ⚡ `form_cache.rs` - Performance
**Rôle :** Optimise les performances via la mise en cache
- Cache des éléments DOM pour éviter les requêtes répétées
- Moniteur de performance
- Gestion intelligente de la mémoire

### ✅ `form_validation.rs` - Validation
**Rôle :** Système de validation complet et extensible
- Règles prédéfinies : obligatoire, email, longueur, motifs
- Validation personnalisée avec fonctions custom
- Messages d'erreur configurables
- Validation en temps réel

**Règles disponibles :**
```rust
ValidationRule::Required              // Champ obligatoire
ValidationRule::Email                 // Format email valide
ValidationRule::MinLength(5)          // Longueur minimale
ValidationRule::MaxLength(100)        // Longueur maximale
ValidationRule::Pattern(regex)        // Expression régulière
ValidationRule::NumberRange(1.0, 10.0) // Plage numérique
ValidationRule::Custom(fonction)      // Validation personnalisée
```

### 🛠️ `form_utilities.rs` - Utilitaires
**Rôle :** Fonctions d'aide et formulaires prêts à l'emploi
- Templates de formulaires courants (contact, inscription, upload)
- Fonctions utilitaires pour manipuler les données
- Helpers pour les tâches répétitives

**Formulaires prêts :**
```rust
contact_form_init("contact", "/api/contact")?;        // Formulaire de contact
registration_form_init("register", "/api/register")?; // Formulaire d'inscription
upload_form_init("upload", "/api/upload")?;          // Formulaire d'upload
```

## Avantages du système

### ✨ Simplicité d'utilisation
- API unifiée avec pattern Builder
- Point d'entrée unique et cohérent

### 🎯 Organisation claire
- Chaque fichier a une responsabilité précise
- Nomenclature cohérente avec préfixe "form_"
- Structure plate et facile à naviguer

### 🚀 Performance améliorée
- Cache DOM intégré
- Validation optimisée
- Gestion mémoire efficace

### 🔧 Extensibilité
- Ajout facile de nouveaux types de champs
- Règles de validation personnalisables
- Configuration flexible

## Exemple complet d'utilisation

Voici un exemple qui utilise toutes les fonctionnalités du système :

```rust
use crate::form::{
    FormHandler, FormConfig, FormField, FieldType, FieldConfig,
    FormValidator, ValidationRule
};
use web_sys::{window, Document};

// 1. CONFIGURATION AVANCÉE
let config = FormConfig::builder()
    .validation(true)                           // Activer la validation
    .auto_focus_error(true)                    // Focus auto sur les erreurs
    .show_loading(true)                        // Afficher l'indicateur de chargement
    .success_message("✅ Formulaire envoyé avec succès !")
    .error_message("❌ Erreur lors de l'envoi")
    .loading_message("📤 Envoi en cours...")
    .max_file_size(10 * 1024 * 1024)         // Taille max 10MB
    .retry_attempts(3)                         // 3 tentatives
    .debounce_ms(300)                         // Validation après 300ms
    .build();

// 2. DÉFINITION DES CHAMPS
let document = window().unwrap().document().unwrap();

// Champ nom (obligatoire, 2-50 caractères)
let nom_element = document.get_element_by_id("nom").unwrap()
    .dyn_into::<web_sys::HtmlInputElement>().unwrap();
let champ_nom = FormField::with_validation(
    "nom".to_string(),
    FieldType::Text,
    nom_element,
    true // obligatoire
)?;

// Champ email (obligatoire, format email)
let email_element = document.get_element_by_id("email").unwrap()
    .dyn_into::<web_sys::HtmlInputElement>().unwrap();
let champ_email = FormField::with_validation(
    "email".to_string(),
    FieldType::Email,
    email_element,
    true
)?;

// Champ téléphone (optionnel, format spécifique)
let tel_element = document.get_element_by_id("telephone").unwrap()
    .dyn_into::<web_sys::HtmlInputElement>().unwrap();
let champ_tel = FormField::with_validation(
    "telephone".to_string(),
    FieldType::Text,
    tel_element,
    false // optionnel
)?;

// Champ message (obligatoire, 10-1000 caractères)
let msg_element = document.get_element_by_id("message").unwrap()
    .dyn_into::<web_sys::HtmlTextAreaElement>().unwrap();
let champ_message = FormField::with_validation(
    "message".to_string(),
    FieldType::TextArea,
    msg_element.into(),
    true
)?;

// Champ fichier (optionnel, max 5MB)
let file_element = document.get_element_by_id("fichier").unwrap()
    .dyn_into::<web_sys::HtmlInputElement>().unwrap();
let champ_fichier = FormField::with_validation(
    "fichier".to_string(),
    FieldType::File,
    file_element,
    false
)?;

// Case à cocher conditions (obligatoire)
let cgu_element = document.get_element_by_id("accepte_cgu").unwrap()
    .dyn_into::<web_sys::HtmlInputElement>().unwrap();
let champ_cgu = FormField::with_validation(
    "accepte_cgu".to_string(),
    FieldType::Checkbox,
    cgu_element,
    true
)?;

let champs = vec![
    champ_nom, champ_email, champ_tel, 
    champ_message, champ_fichier, champ_cgu
];

// 3. VALIDATION PERSONNALISÉE
let validator = FormValidator::new()
    // Validation du nom (2-50 caractères, pas de chiffres)
    .add_rule("nom", ValidationRule::text(2, 50))
    .add_rule("nom", ValidationRule::custom(|value| {
        if value.chars().any(|c| c.is_ascii_digit()) {
            Err("Le nom ne peut pas contenir de chiffres".to_string())
        } else {
            Ok(())
        }
    }))
    
    // Validation de l'email
    .add_rule("email", ValidationRule::email())
    
    // Validation du téléphone (format français optionnel)
    .add_rule("telephone", ValidationRule::pattern(
        r"^(?:\+33|0)[1-9](?:[0-9]{8})$",
        "Format : +33123456789 ou 0123456789"
    ))
    
    // Validation du message
    .add_rule("message", ValidationRule::text(10, 1000))
    
    // Validation du fichier (images seulement, max 5MB)
    .add_rule("fichier", ValidationRule::file_size(5 * 1024 * 1024))
    .add_rule("fichier", ValidationRule::custom(|value| {
        if value.is_empty() { return Ok(()); } // optionnel
        
        let extensions_autorisees = ["jpg", "jpeg", "png", "gif", "webp"];
        let extension = value.split('.').last().unwrap_or("").to_lowercase();
        
        if extensions_autorisees.contains(&extension.as_str()) {
            Ok(())
        } else {
            Err("Seules les images sont autorisées (jpg, png, gif, webp)".to_string())
        }
    }))
    
    // Validation CGU
    .add_rule("accepte_cgu", ValidationRule::required());

// 4. CRÉATION DU GESTIONNAIRE DE FORMULAIRE
let handler = FormHandler::new("formulaire-contact", "/api/contact", config)?
    .with_field_configs(&champs)
    .with_validator(validator)
    .build()?;

// 5. CALLBACKS PERSONNALISÉS (optionnel)
handler.on_before_submit(|data| {
    console::log_1(&"📋 Données avant envoi :".into());
    console::log_1(&format!("{:?}", data).into());
    true // Continuer l'envoi
})?;

handler.on_success(|response| {
    console::log_1(&"🎉 Succès !".into());
    // Redirection ou autre action
    window().unwrap().location().set_href("/merci")?;
    Ok(())
})?;

handler.on_error(|error| {
    console::log_1(&format!("💥 Erreur : {}", error).into());
    // Logging ou autre action
    Ok(())
})?;

// 6. INITIALISATION
handler.initialize()?;

// 7. UTILISATION AVANCÉE

// Validation manuelle d'un champ
if let Err(erreurs) = handler.validate_field("email") {
    for erreur in erreurs {
        console::log_1(&format!("Erreur email : {}", erreur).into());
    }
}

// Récupération des données du formulaire
let donnees = handler.get_form_data()?;
console::log_1(&format!("Données actuelles : {:?}", donnees).into());

// Pré-remplissage du formulaire
let donnees_initiales = serde_json::json!({
    "nom": "Jean Dupont",
    "email": "jean.dupont@example.com"
});
handler.set_form_data(&donnees_initiales)?;

// Réinitialisation du formulaire
handler.reset_form()?;

// Activation/désactivation de champs
handler.set_field_disabled("email", true)?;
handler.set_field_disabled("email", false)?;

// Mise à jour des messages d'erreur
handler.show_field_error("nom", "Ce nom est déjà utilisé")?;
handler.clear_field_error("nom")?;

// Soumission programmatique
handler.submit_form().await?;
```

## Cas d'usage typiques

### 📧 Formulaire de contact simple
```rust
use crate::form::utilities::contact_form_init;

// Création rapide d'un formulaire de contact
contact_form_init("contact", "/api/contact")?;
```

### 👤 Formulaire d'inscription complet
```rust
use crate::form::utilities::registration_form_init;

// Formulaire d'inscription avec validation avancée
registration_form_init("inscription", "/api/register")?;
```

### 📎 Upload de fichiers
```rust
use crate::form::utilities::upload_form_init;

// Formulaire d'upload avec gestion des fichiers
upload_form_init("upload", "/api/files")?;
```

### 🎨 Formulaire personnalisé
```rust
// Pour des besoins spécifiques, utiliser l'API complète
let handler = FormHandler::new("custom", "/api/custom", config)?
    .with_field_specs(&[
        ("titre", FieldType::Text),
        ("contenu", FieldType::TextArea),
        ("public", FieldType::Checkbox),
    ])
    .build()?;
```

## Conseils d'utilisation

### 🎯 Bonnes pratiques
1. **Utilisez les utilitaires** pour les cas simples
2. **Configurez la validation** selon vos besoins
3. **Gérez les erreurs** de manière user-friendly
4. **Testez sur différents navigateurs**
5. **Optimisez les performances** avec le cache

### ⚠️ Points d'attention
- Les champs doivent exister dans le DOM avant l'initialisation
- La validation côté client ne remplace pas la validation serveur
- Gérez les cas d'erreur réseau
- Pensez à l'accessibilité (ARIA, navigation clavier)

### 🔧 Dépannage
- **Erreur "Element not found"** : Vérifiez les IDs des éléments HTML
- **Validation qui ne fonctionne pas** : Vérifiez la configuration et les règles
- **Problème de performance** : Activez le cache et optimisez les règles de validation

Ce système offre une solution complète et flexible pour tous vos besoins de formulaires dans les applications WebAssembly ! 🚀
