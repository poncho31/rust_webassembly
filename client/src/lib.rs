use core::User;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    window, Document, HtmlInputElement,
    Request, RequestInit, RequestMode, Event, File, FormData,
};
use serde_json::json;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

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
    opts.set_body(&body_js);  // Correction ici
    
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

#[derive(Deserialize)]
struct PingResponse {
    status: String,
    message: String,
}

async fn ping_server() {
    match fetch_json::<PingResponse>(&format!("{}/ping", API_BASE_URL)).await {
        Ok(response) => log(&format!("Réponse du serveur: {} - {}", response.status, response.message)),
        Err(e) => log(&format!("Failed to ping server: {:?}", e)),
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

// ==================== Generic Functions & Types ====================
#[derive(Default)]
struct RequestParams {
    timeout: Option<u32>,
    retry_count: Option<u32>,
    cache: bool,
    headers: Vec<(String, String)>,
    content_type: Option<String>,
}

enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl ToString for HttpMethod {
    fn to_string(&self) -> String {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
        }.to_string()
    }
}

// Fonction de base pour toutes les requêtes
async fn make_request(
    url: &str,
    method: HttpMethod,
    params: RequestParams,
    body: Option<&JsValue>
) -> Result<web_sys::Response, JsValue> {
    let window = window().unwrap();
    let opts = RequestInit::new();
    opts.set_method(&method.to_string());
    opts.set_mode(RequestMode::Cors);
    
    if let Some(body_content) = body {
        opts.set_body(body_content);  // Correction ici
    }

    let request = Request::new_with_str_and_init(url, &opts)?;
    
    // Headers par défaut
    request.headers().set("Accept", "application/json")?;
    
    // Content-type personnalisé ou par défaut
    let content_type = params.content_type.unwrap_or_else(|| "application/json".to_string());
    request.headers().set("Content-Type", &content_type)?;
    
    // Headers personnalisés
    for (key, value) in params.headers {
        request.headers().set(&key, &value)?;
    }

    if !params.cache {
        request.headers().set("Cache-Control", "no-cache")?;
    }

    let resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    resp.dyn_into()
}

// Fonctions spécialisées
async fn fetch_json<T>(url: &str) -> Result<T, JsValue> 
where T: for<'a> serde::Deserialize<'a> {
    let params = RequestParams {
        timeout: Some(5000),
        cache: true,
        ..Default::default()
    };

    let resp = make_request(url, HttpMethod::GET, params, None).await?;
    let json = JsFuture::from(resp.json()?).await?;
    Ok(serde_wasm_bindgen::from_value(json)?)
}

async fn fetch_raw(url: &str, method: HttpMethod) -> Result<String, JsValue> {
    let params = RequestParams {
        timeout: Some(5000),
        cache: false,
        ..Default::default()
    };

    let resp = make_request(url, method, params, None).await?;
    let text = JsFuture::from(resp.text()?).await?;
    Ok(text.as_string().unwrap_or_default())
}

async fn post_json<T>(url: &str, data: &T) -> Result<(), JsValue> 
where T: serde::Serialize {
    let params = RequestParams {
        timeout: Some(5000),
        ..Default::default()
    };

    let body = serde_json::to_string(data).map_err(|e| e.to_string())?;
    let body_js = JsValue::from_str(&body);

    let _ = make_request(url, HttpMethod::POST, params, Some(&body_js)).await?;
    Ok(())
}

async fn delete_resource(url: &str) -> Result<(), JsValue> {
    let params = RequestParams::default();
    let _ = make_request(url, HttpMethod::DELETE, params, None).await?;
    Ok(())
}

// Téléchargement de fichiers
async fn download_file(url: &str) -> Result<web_sys::Blob, JsValue> {
    let params = RequestParams {
        timeout: Some(30000),
        content_type: Some("application/octet-stream".to_string()),
        ..Default::default()
    };

    let resp = make_request(url, HttpMethod::GET, params, None).await?;
    JsFuture::from(resp.blob()?).await?.dyn_into()
}

// Upload de fichiers
async fn upload_file(url: &str, file: File) -> Result<(), JsValue> {
    let form_data = FormData::new()?;
    form_data.append_with_str("file", &file.name())?;

    let params = RequestParams {
        timeout: Some(30000),
        content_type: Some("multipart/form-data".to_string()),
        ..Default::default()
    };

    let _ = make_request(url, HttpMethod::POST, params, Some(&form_data.into())).await?;
    Ok(())
}

// Requête avec retry automatique
async fn fetch_with_retry<F, Fut, T>(
    retries: u32,
    operation: F
) -> Result<T, JsValue>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, JsValue>>,
{
    let mut attempts = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts >= retries {
                    return Err(e);
                }
                // Exponential backoff
                let delay = 1000 * (2_u32.pow(attempts - 1));
                sleep(delay).await;
            }
        }
    }
}

// Utilitaire pour le sleep
async fn sleep(ms: u32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                ms as i32,
            )
            .unwrap();
    });
    JsFuture::from(promise).await.unwrap();
}

// Exemple d'utilisation:
#[allow(dead_code)]
async fn example_usage() {
    // Récupérer un utilisateur avec type explicite
    let _user: User = fetch_json::<User>(&format!("{}/users/1", API_BASE_URL))
        .await
        .unwrap();
    
    // Créer un utilisateur
    let new_user = User {
        id: "".to_string(),
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    post_json(&format!("{}/users", API_BASE_URL), &new_user).await.unwrap();
    
    // Télécharger un fichier
    let _blob = download_file(&format!("{}/files/document.pdf", API_BASE_URL))
        .await
        .unwrap();
    
    // Avec retry automatique
    let _result = fetch_with_retry::<_, _, User>(3, || async {
        fetch_json::<User>(&format!("{}/users/1", API_BASE_URL)).await
    }).await;
}
