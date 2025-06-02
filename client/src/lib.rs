mod client_tools;
mod client_request;
mod client_periodics;
pub mod form;
pub mod client_form_improved;
mod validation;
pub mod modal;
pub mod refresh;

use wasm_bindgen::prelude::*;
use form::{
    handler::FormHandler,
    config::FormConfig,
    field::{FieldType, FieldConfig, FieldOption},
};
use client_form_improved::{form_init_with_config};
use validation::{FormValidator, ValidationRule};
use client_tools::log;
use refresh::{RefreshConfig, RefreshScheduler};
use refresh::config::DataTransform;

// Fonction d'initialisation du script
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    log("# Begin script - Enhanced Form System");

    // Initialiser les rafraîchissements automatiques
    log("# Initializing auto-refresh system");
    wasm_bindgen_futures::spawn_local(init_auto_refresh());// Configuration avancée pour le formulaire principal
    log("# Enhanced form initialization");
    
    // Configuration du formulaire principal avec validation avancée
    let main_form_config = FormConfig::builder()
        .validation(true)
        .retry_attempts(3)
        .loading(true)
        .auto_focus_error(true)
        .success_message("✅ Formulaire soumis avec succès!")
        .error_message("❌ Erreur lors de la soumission")
        .max_file_size(10 * 1024 * 1024) // 10MB max pour les fichiers
        .build();    // Création du validateur avec règles personnalisées
    let validator = FormValidator::new()
        .add_rule("login", ValidationRule::text(3, 20))
        .add_rule("firstname", ValidationRule::text(2, 50)) 
        .add_rule("lastname", ValidationRule::text(2, 50))
        .add_rule("email", ValidationRule::email())
        .add_rule("age", ValidationRule::number(0.0, 150.0));

    // Configuration des champs avec options et valeurs par défaut
    use std::collections::HashMap;
    let mut field_configs = HashMap::new();
    
    // Configuration pour le champ sexe avec options
    let sexe_options = vec![
        FieldOption::new("", "Sélectionnez..."),
        FieldOption::new("homme", "Homme"),
        FieldOption::new("femme", "Femme"),
        FieldOption::new("autre", "Autre"),
    ];
    field_configs.insert("sexe", FieldConfig::new(FieldType::Select)
        .with_options(sexe_options)
        .required());

    // Configuration pour les autres champs
    field_configs.insert("login", FieldConfig::new(FieldType::Text)
        .with_placeholder("Votre identifiant")
        .required());
        
    field_configs.insert("info", FieldConfig::new(FieldType::TextArea)
        .with_placeholder("Informations supplémentaires (optionnel)"));
        
    field_configs.insert("birthday", FieldConfig::new(FieldType::Date)
        .required());
        
    field_configs.insert("firstname", FieldConfig::new(FieldType::Text)
        .with_placeholder("Votre prénom")
        .required());
        
    field_configs.insert("lastname", FieldConfig::new(FieldType::Text)
        .with_placeholder("Votre nom")
        .required());
        
    field_configs.insert("email", FieldConfig::new(FieldType::Email)
        .with_placeholder("votre@email.com")
        .required());
        
    field_configs.insert("files", FieldConfig::new(FieldType::File));
      field_configs.insert("age", FieldConfig::new(FieldType::Number)
        .with_placeholder("Âge")
        .required());

    // Initialisation du formulaire principal avec la nouvelle API et configurations de champs
    match FormHandler::new_with_field_configs("form", "/api/form", Some(&field_configs), main_form_config) {
        Ok(handler) => {
            match handler.with_validator(validator).initialize() {
                Ok(_) => log("✅ Formulaire principal initialisé avec succès avec configurations de champs"),
                Err(e) => {
                    log(&format!("❌ Erreur lors de l'initialisation du formulaire principal: {:?}", e));
                    return Err(e);
                }
            }
        },
        Err(e) => {
            log(&format!("❌ Erreur lors de la création du handler: {:?}", e));
            return Err(e);
        }
    }// Configuration simple pour le bouton ping
    let ping_config = FormConfig::builder()
        .validation(false)
        .loading(true)
        .success_message("🏓 Ping envoyé!")
        .build();    // Initialisation du formulaire ping avec la nouvelle API simplifiée
    match form_init_with_config("button_ping", "/api/ping", None, ping_config) {
        Ok(_) => log("✅ Bouton ping initialisé avec succès"),
        Err(e) => {
            log(&format!("❌ Erreur lors de l'initialisation du bouton ping: {:?}", e));
            return Err(e);
        }
    }    log("# End script - Enhanced Form System with Auto-Refresh Ready");
    Ok(())
}

/// Initialise les rafraîchissements automatiques pour les exemples dans index.html
async fn init_auto_refresh() {
    log("🔄 Configuration des rafraîchissements automatiques");    // Configuration pour la température actuelle (basée sur le champ région)
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
    }).with_input_field("#region"); // Utilise le champ région comme paramètre



    let counter_config = RefreshConfig::new_text(
        "status_code",
        "/api/ping",
        10,  // Toutes les 10 secondes
        "#auto-counter",
        Some("status"),
    ).with_transform(DataTransform {
        prefix: Some("Code de statut: ".to_string()),
        suffix: None,
        format: Some("number".to_string()),
    });

    // Configuration pour un message HTML
    let message_config = RefreshConfig::new_html(
        "message",
        "/api/ping",
        60,  // Toutes les minutes
        "#auto-message",
        Some("message"),
    ).with_transform(DataTransform {
        prefix: Some("<strong>".to_string()),
        suffix: Some("</strong>".to_string()),
        format: None,
    });    // Démarrer tous les rafraîchissements
    RefreshScheduler::new()
        .add_refresh(temperature_config)
        .add_refresh(counter_config)
        .add_refresh(message_config)
        .start_all();

    log("✅ Système de rafraîchissement automatique démarré");
}

