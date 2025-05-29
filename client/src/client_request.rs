use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, window, File, FormData};
use serde::{Serialize, Deserialize};
use core::http_models::http_responses::HttpSendResponse;

#[allow(dead_code)]
#[derive(Default)]
pub struct RequestParams {
    pub timeout: Option<u32>,
    pub retry_count: Option<u32>,
    pub cache: bool,
    pub headers: Vec<(String, String)>,
    pub content_type: Option<String>,
}

#[allow(dead_code)]
pub enum HttpMethod {
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


pub async fn make_request(
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
        opts.set_body(body_content);
    }

    let request = Request::new_with_str_and_init(url, &opts)?;
    
    request.headers().set("Accept", "application/json")?;
    
    let content_type = params.content_type.unwrap_or_else(|| "application/json".to_string());
    request.headers().set("Content-Type", &content_type)?;
    
    for (key, value) in params.headers {
        request.headers().set(&key, &value)?;
    }

    if !params.cache {
        request.headers().set("Cache-Control", "no-cache")?;
    }

    let resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    resp.dyn_into()
}

#[allow(dead_code)]
pub async fn fetch_json<T>(url: &str) -> Result<T, JsValue> 
where T: for<'a> Deserialize<'a> {
    let params = RequestParams {
        timeout: Some(5000),
        cache: true,
        ..Default::default()
    };

    let resp = make_request(url, HttpMethod::GET, params, None).await?;
    let json = JsFuture::from(resp.json()?).await?;
    Ok(serde_wasm_bindgen::from_value(json)?)
}

#[allow(dead_code)]
pub async fn fetch_raw(url: &str, method: HttpMethod) -> Result<String, JsValue> {
    let params = RequestParams {
        timeout: Some(5000),
        cache: false,
        ..Default::default()
    };

    let resp = make_request(url, method, params, None).await?;
    let text = JsFuture::from(resp.text()?).await?;
    Ok(text.as_string().unwrap_or_default())
}

#[allow(dead_code)]
pub async fn post_json<T>(url: &str, data: &T) -> Result<(), JsValue> 
where T: Serialize {
    let params = RequestParams {
        timeout: Some(5000),
        ..Default::default()
    };

    let body = serde_json::to_string(data).map_err(|e| e.to_string())?;
    let body_js = JsValue::from_str(&body);

    let _ = make_request(url, HttpMethod::POST, params, Some(&body_js)).await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn delete_resource(url: &str) -> Result<(), JsValue> {
    let params = RequestParams::default();
    let _ = make_request(url, HttpMethod::DELETE, params, None).await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn download_file(url: &str) -> Result<web_sys::Blob, JsValue> {
    let params = RequestParams {
        timeout: Some(30000),
        content_type: Some("application/octet-stream".to_string()),
        ..Default::default()
    };

    let resp = make_request(url, HttpMethod::GET, params, None).await?;
    JsFuture::from(resp.blob()?).await?.dyn_into()
}

#[allow(dead_code)]
pub async fn upload_file(url: &str, file: File) -> Result<(), JsValue> {
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

#[allow(dead_code)]
pub async fn fetch_with_retry<F, Fut, T>(
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
                let delay = 1000 * (2_u32.pow(attempts - 1));
                sleep(delay).await;
            }
        }
    }
}

#[allow(dead_code)]
pub async fn sleep(ms: u32) {
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

pub async fn post_form(endpoint: &str, form_data: &web_sys::FormData) -> Result<HttpSendResponse, JsValue> {
    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::Cors);
    
    let form_data_js: JsValue = form_data.clone().into();
    opts.set_body(&form_data_js);  // Modification ici : on passe directement la référence

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
