mod client_tools;
mod client_request;
mod client_periodics;
pub mod form;
pub mod client_form_improved;
mod validation;
pub mod modal;
pub mod refresh;

use core::{http_models::http_responses::HttpSendResponse};
use wasm_bindgen::prelude::*;
use web_sys::{window};
use form::{
    handler::FormHandler,
    config::FormConfig,
    field::FieldType,
};
use client_form_improved::{form_init_with_config};
use validation::{FormValidator, ValidationRule};
use client_request::fetch_json;
use client_periodics::run_async_request;
use client_tools::log;
use refresh::{RefreshConfig, RefreshScheduler};
use refresh::config::DataTransform;

const API_BASE_URL: &str = "api";

/// Met à jour l'interface utilisateur avec le statut du serveur et le message
/// - status: Le statut à afficher (OK, Error, etc.)
/// - message: Le message détaillé à afficher
fn update_status_display(status: &str, message: &str) {
    let window = window().unwrap();
    let document = window.document().unwrap();
 
    // Met à jour l'élément affichant le statut
    if let Some(status_el) = document.get_element_by_id("server-status") {
        status_el.set_text_content(Some(status));
    }
    // Met à jour l'élément affichant le message
    if let Some(message_el) = document.get_element_by_id("server-message") {
        message_el.set_text_content(Some(message));
    }
}

/// Ping_server utilisant run_async_request
async fn ping_server(interval_seconds: i32) {
    let do_ping = || async {
        match fetch_json::<HttpSendResponse>(&format!("{}/ping", API_BASE_URL)).await {
            Ok(response) => {
                let message = response.message.unwrap_or_default();
                let status = response.status.to_string();
                update_status_display(&status, &message);
                log(&format!("Ping response: status={}, message={}", status, message));
            },
            Err(e) => {
                let error_msg = format!("Failed to ping server: {:?}", e);
                update_status_display("Error", &error_msg);
                log(&error_msg);
            },
        }
    };

    run_async_request(do_ping, interval_seconds).await;
}



// Fonction d'initialisation du script
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    log("# Begin script - Enhanced Form System");

    // Ping server every 60 seconds
    log("# Ping server every 60 seconds");
    wasm_bindgen_futures::spawn_local(ping_server(60));

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
        .build();

    // Création du validateur avec règles personnalisées
    let validator = FormValidator::new()
        .add_rule("login", ValidationRule::text(3, 20))
        .add_rule("firstname", ValidationRule::text(2, 50)) 
        .add_rule("lastname", ValidationRule::text(2, 50))
        .add_rule("email", ValidationRule::email())
        .add_rule("age", ValidationRule::number(0.0, 150.0));

    // Spécifications des champs avec types appropriés
    let main_form_fields = &[
        ("login", FieldType::Text),
        ("birthday", FieldType::Date),
        ("firstname", FieldType::Text),
        ("lastname", FieldType::Text),
        ("email", FieldType::Email),
        ("files", FieldType::File),
        ("age", FieldType::Number),
    ];    // Initialisation du formulaire principal avec la nouvelle API
    match FormHandler::new("form", "/api/form", Some(main_form_fields), main_form_config) {
        Ok(handler) => {
            match handler.with_validator(validator).initialize() {
                Ok(_) => log("✅ Formulaire principal initialisé avec succès"),
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
    log("🔄 Configuration des rafraîchissements automatiques");    // Configuration pour le statut du serveur (affichage simple)
    let server_status_config = RefreshConfig::new_text(
        "server_status",
        "/api/ping", 
        30,  // Toutes les 30 secondes
        "#auto-server-status",
        Some("status"),
    ).with_transform(DataTransform {
        prefix: Some("HTTP ".to_string()),
        suffix: None,
        format: Some("number".to_string()),
    });// Configuration pour le code de statut (exemple)
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
    });

    // Démarrer tous les rafraîchissements
    RefreshScheduler::new()
        .add_refresh(server_status_config)
        .add_refresh(counter_config)
        .add_refresh(message_config)
        .start_all();

    log("✅ Système de rafraîchissement automatique démarré");
}

