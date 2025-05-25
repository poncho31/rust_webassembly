use core::User;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    window, Document, HtmlInputElement,
    Request, RequestInit, RequestMode, Event,
};
use serde_json::json;

// Utiliser un chemin relatif puisque le client et l'API sont sur le même domaine
const API_BASE_URL: &str = "api";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

async fn fetch_users() -> Result<Vec<User>, JsValue> {
    let window = window().unwrap();



    
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);
    
    let request = Request::new_with_str_and_init(
        &format!("{}/users", API_BASE_URL),
        &opts
    )?;
    
    request.headers().set("Accept", "application/json")?;
    request.headers().set("Content-Type", "application/json")?;
    
    let resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;
    let users: Vec<User> = serde_wasm_bindgen::from_value(json)?;
    Ok(users)
}

async fn create_user(name: String, email: String) -> Result<(), JsValue> {
    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::Cors);
    
    let body = json!({
        "name": name,
        "email": email
    }).to_string();
    
    let body_js = JsValue::from_str(&body);
    opts.set_body(&body_js);  // Passer directement la référence sans Some()
    

    let request = Request::new_with_str_and_init(
        &format!("{}/users", API_BASE_URL),
        &opts
    )?;

    request.headers().set("Accept", "application/json")?;
    request.headers().set("Content-Type", "application/json")?;
    
    let window = window().unwrap();
    let resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    let _: web_sys::Response = resp.dyn_into()?;

    Ok(())



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
