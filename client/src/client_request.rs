use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, window};
use serde::Deserialize;
use core::http_models::http_responses::HttpSendResponse;

/// Fetch JSON data from a URL
pub async fn fetch_json<T>(url: &str) -> Result<T, JsValue> 
where T: for<'a> Deserialize<'a> {
    let window = window().unwrap();
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)?;
    request.headers().set("Accept", "application/json")?;
    request.headers().set("Content-Type", "application/json")?;
    request.headers().set("Cache-Control", "no-cache")?;

    let resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: web_sys::Response = resp.dyn_into()?;
    
    let json = JsFuture::from(response.json()?).await?;
    Ok(serde_wasm_bindgen::from_value(json)?)
}

/// Submit form data via POST
pub async fn post_form(endpoint: &str, form_data: &web_sys::FormData) -> Result<HttpSendResponse, JsValue> {
    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::Cors);
    
    let form_data_js: JsValue = form_data.clone().into();
    opts.set_body(&form_data_js);

    let request = Request::new_with_str_and_init(endpoint, &opts)?;
    
    let window = web_sys::window().unwrap();
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: web_sys::Response = response.dyn_into()?;
    
    if !response.ok() {
        return Err(JsValue::from_str(&format!("HTTP error! status: {}", response.status())));
    }
    
    let json = JsFuture::from(response.json()?).await?;
    let response_data: HttpSendResponse = serde_wasm_bindgen::from_value(json)?;
    Ok(response_data)
}
