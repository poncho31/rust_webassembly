mod client_tools;
mod client_request;
mod client_periodics;
mod client_form;

use core::{http_models::http_responses::HttpSendResponse};
use wasm_bindgen::prelude::*;
use web_sys::{window};
use client_form::{form_init, FieldType};
use client_request::fetch_json;
use client_periodics::run_async_request;
use client_tools::log;

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
            },
            Err(e) => {
                let error_msg = format!("Failed to ping server: {:?}", e);
                update_status_display("Error", &error_msg);
            },
        }
    };

    run_async_request(do_ping, interval_seconds).await;
}

// Fonction d'initialisation du script
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    log("# Begin script");

    // Ping server
    log("# Ping server every 5 seconds");
    wasm_bindgen_futures::spawn_local(ping_server(5));

    // Form init
    log("# Send form initialization");
    form_init("form", "/api/form",&[
        ("login",     FieldType::Text),
        ("firstname", FieldType::Text),
        ("lastname",  FieldType::Text),
        ("email",     FieldType::Text),
        ("files",     FieldType::File),
        ("age",       FieldType::Text),
    ])?;

    // form_init("button", &[("button", FieldType::Text)])?;

    log("# End script");
    Ok(())
}

