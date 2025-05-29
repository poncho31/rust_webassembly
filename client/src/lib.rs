mod client_request;
mod client_periodics;

use core::{http_models::http_responses::HttpSendResponse};
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlInputElement, Event, Document, Element, Window};
use client_request::fetch_json;
use client_periodics::run_async_request;

const API_BASE_URL: &str = "api";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

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

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    log("#Begin script");

    // Ping server
    log("#Ping server every 5 seconds");
    wasm_bindgen_futures::spawn_local(ping_server(5));

    // Form init
    log("#Send form initialization");
    form_init("form")?;

    log("#End script");
    Ok(())

}


fn form_init(form_id : &str)-> Result<(), JsValue> {
    log("##Debut script send form");

    // Récupère la fenêtre et le document
    let window     : Window           = window().unwrap();
    let document   : Document         = window.document().unwrap();
    let form       : Element          = document.get_element_by_id(&form_id).ok_or_else(|| JsValue::from_str("Form not found"))?;
    let name_input : HtmlInputElement = input(&document, "name")?;
    let email_input: HtmlInputElement = input(&document, "email")?;


    let closure = Closure::wrap(Box::new(move |e: Event| {
        e.prevent_default();
        let _name  = name_input.value();
        let _email = email_input.value();
        log("####Formulaire envoyé");

    }) as Box<dyn FnMut(_)>);

    form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())?;
    closure.forget();

    log("####Fin script send form");
    Ok(())
}

fn input(document: &Document, element_name: &str) -> Result<HtmlInputElement, JsValue> {
    Ok(document
        .get_element_by_id(element_name)
        .ok_or_else(|| JsValue::from_str(&format!("Input element {} not found", element_name)))?
        .dyn_into::<HtmlInputElement>()?)
}