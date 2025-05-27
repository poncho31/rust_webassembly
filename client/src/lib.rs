mod client_request;

use core::{http_models::http_responses::HttpSendResponse};
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlInputElement, Event};
use client_request::{fetch_json};

const API_BASE_URL: &str = "api";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

async fn ping_server() {
    match fetch_json::<HttpSendResponse>(&format!("{}/ping", API_BASE_URL)).await {
        Ok(response) => log(&format!("{} - {}", response.status, response.message.unwrap_or_default())),
        Err(e)       => log(&format!("Failed to ping server: {:?}", e)),
    }
}


#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    // Test the server connection at startup
    wasm_bindgen_futures::spawn_local(ping_server());

    let window = window().unwrap();
    let document = window.document().unwrap();

    // Setup form handler
    let form = document.get_element_by_id("form").unwrap();
    let name_input: HtmlInputElement = document
        .get_element_by_id("name")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let email_input: HtmlInputElement = document
        .get_element_by_id("email")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();

    let name_clone = name_input.clone();
    let email_clone = email_input.clone();

    let _document_clone = document.clone();
    let closure = Closure::wrap(Box::new(move |e: Event| {
        e.prevent_default();
        let _name = name_clone.value();
        let _email = email_clone.value();

    }) as Box<dyn FnMut(_)>);

    form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())?;
    closure.forget();

    // Initial load - just ping the server
    wasm_bindgen_futures::spawn_local(ping_server());

    Ok(())
}
