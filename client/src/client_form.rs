use web_sys::{window, HtmlInputElement, HtmlButtonElement, Event, Document, FormData};
use wasm_bindgen::prelude::*;
use crate::{client_tools::log, client_request, modal::Modal};
use serde_json::{Value, json};
use gloo_timers::callback::Timeout;

#[derive(Clone)]
pub enum FieldType { Text, File }

#[derive(Clone)]
pub struct FormField {
    id: String,
    field_type: FieldType,
    input: HtmlInputElement,
}

pub fn form_init(form_id: &str, endpoint: &str, field_specs: Option<&[(&str, FieldType)]>) -> Result<(), JsValue> {
    let document = window().unwrap().document().unwrap();
    let form = document.get_element_by_id(form_id)
        .ok_or_else(|| JsValue::from_str("Form not found"))?;

    let fields = create_form_fields(&document, field_specs)?;
    let submit_button = document.query_selector(&format!("button[form='{}']", form_id))?
        .ok_or_else(|| JsValue::from_str("Submit button not found"))?
        .dyn_into::<HtmlButtonElement>()?;
    let modal = Modal::new()?;

    let closure = create_submit_handler(endpoint.to_string(), fields, modal, submit_button);
    form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}

fn create_submit_handler(endpoint: String, fields: Vec<FormField>, modal: Modal, button: HtmlButtonElement) -> Closure<dyn FnMut(Event)> {
    Closure::wrap(Box::new(move |e: Event| {
        e.prevent_default();
        let (endpoint, fields, modal, button) = (endpoint.clone(), fields.clone(), modal.clone(), button.clone());
        
        wasm_bindgen_futures::spawn_local(async move {
            let (form_data, debug_data) = process_form_data(&fields).await;
            log_form(&endpoint, &debug_data);
            submit_form_data(&endpoint, form_data, &modal, &button).await;
        });
    }) as Box<dyn FnMut(_)>)
}

fn create_form_fields(document: &Document, specs: Option<&[(&str, FieldType)]>) -> Result<Vec<FormField>, JsValue> {
    let mut fields = Vec::new();
    if let Some(specs) = specs {
        for &(id, ref field_type) in specs {
            let input = document.get_element_by_id(id)
                .ok_or_else(|| JsValue::from_str(&format!("Input {} not found", id)))?
                .dyn_into::<HtmlInputElement>()?;

            if matches!(field_type, FieldType::File) {
                input.set_attribute("type", "file")?;
                input.set_attribute("multiple", "")?;
            }

            fields.push(FormField { id: id.to_string(), field_type: field_type.clone(), input });
        }
    }
    Ok(fields)
}

async fn submit_form_data(endpoint: &str, form_data: FormData, modal: &Modal, button: &HtmlButtonElement) {
    button.set_inner_html(&format!("{}<div class='loader'></div>", button.inner_text()));
    
    let result = match client_request::post_form(endpoint, &form_data).await {
        Ok(response) if response.is_success() => format!("✓ {}", response.get_message()),
        Ok(response) => format!("⨯ Erreur: {}", response.get_message()),
        Err(e) => format!("⨯ Erreur: {:?}", e),
    };

    modal.show(&result).unwrap();
    button.set_inner_text(&button.inner_text().replace("Submit", "Submit"));
}

async fn process_form_data(fields: &[FormField]) -> (FormData, Value) {
    let form_data = FormData::new().unwrap();
    let mut debug_data = json!({});

    for field in fields {
        match field.field_type {
            FieldType::Text => {
                let value = field.input.value();
                form_data.append_with_str(&field.id, &value).unwrap();
                debug_data[&field.id] = Value::String(value);
            }
            FieldType::File => if let Some(files) = field.input.files() {
                let mut files_info = Vec::new();
                for i in 0..files.length() {
                    if let Some(file) = files.item(i) {
                        form_data.append_with_blob(&field.id, &file).unwrap();
                        files_info.push(format!("{}({} bytes)", file.name(), file.size()));
                    }
                }
                debug_data[&field.id] = json!(files_info);
            }
        }
    }
    (form_data, debug_data)
}

async fn delay_timeout(duration_ms: u32) {
    let (sender, receiver) = futures::channel::oneshot::channel::<()>();
    let _timeout = Timeout::new(duration_ms, move || {
        let _ = sender.send(());
    });
    receiver.await.unwrap();
}

fn log_form(endpoint: &str, debug_data: &Value) {
    log("=== Client Form Data ===");
    log(&format!("Endpoint: {}", endpoint));
    if !debug_data.as_object().unwrap().is_empty() {
        log(&format!("Content: {}", debug_data.to_string()));
    } else {
        log("No form fields to send");
    }
    log("===============");
}