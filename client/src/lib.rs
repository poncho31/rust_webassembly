mod client_request;

use core::{User, http_responses::HttpSendResponse};
use wasm_bindgen::prelude::*;
use web_sys::{window, Document, HtmlInputElement, Event};
use serde_json::json;
use client_request::{fetch_json, post_json};

const API_BASE_URL: &str = "api";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

async fn fetch_users() -> Result<Vec<User>, JsValue> {
    fetch_json(&format!("{}/users", API_BASE_URL)).await
}

async fn create_user(name: String, email: String) -> Result<(), JsValue> {
    let new_user = json!({
        "name": name,
        "email": email
    });
    post_json(&format!("{}/users", API_BASE_URL), &new_user).await
}

async fn ping_server() {
    match fetch_json::<HttpSendResponse>(&format!("{}/ping", API_BASE_URL)).await {
        Ok(response) => log(&format!("{} - {}", response.status, response.message)),
        Err(e)       => log(&format!("Failed to ping server: {:?}", e)),
    }
}

fn display_users(doc: &Document, users: Vec<User>) -> Result<(), JsValue> {
    let users_div = doc.get_element_by_id("users").unwrap();
    users_div.set_inner_html("");

    for user in users {
        let user_el = doc.create_element("div")?;
        user_el.set_class_name("user-item");
        user_el.set_inner_html(&format!(
            "<strong>{}</strong> ({})", 
            user.name, 
            user.email
        ));
        users_div.append_child(&user_el)?;
    }
    Ok(())
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

    let document_clone = document.clone();
    let closure = Closure::wrap(Box::new(move |e: Event| {
        e.prevent_default();
        let name = name_clone.value();
        let email = email_clone.value();

        // Use cloned document instead of getting new window reference
        let doc = document_clone.clone();

        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(()) = create_user(name, email).await {
                if let Ok(users) = fetch_users().await {
                    display_users(&doc, users).unwrap();
                }
            }
        });
    }) as Box<dyn FnMut(_)>);

    form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())?;
    closure.forget();

    // Initial load
    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(users) = fetch_users().await {
            display_users(&document, users).unwrap();
        }
    });

    Ok(())
}
