use web_sys::{window, console, HtmlInputElement, HtmlButtonElement, Event, Document, Element, Window, FormData};
use wasm_bindgen::prelude::*;
use crate::client_tools::log;
use crate::client_request;
use crate::modal::Modal;
use serde_json::{Value, json};
use gloo_timers::callback::Timeout;

#[derive(Clone)]
pub enum FieldType {
    Text,
    File,
}

/// Structure représentant un champ de formulaire
#[derive(Clone)]
pub struct FormField {
    id         : String,
    field_type : FieldType,
    input      : HtmlInputElement,
}


pub fn form_init(form_id: &str, endpoint: &str, field_specs: Option<&[(&str, FieldType)]>) -> Result<(), JsValue> {
    log("#### Debut script send form");

    let window   = window().unwrap();
    let document = window.document().unwrap();
    let form     = document.get_element_by_id(form_id).ok_or_else(|| JsValue::from_str(&format!("Form id not found : #{}", form_id)))?;

    let fields        = create_form_fields(&document, field_specs)?;
    let submit_button = find_submit_button(&form, &document, form_id)?;
    let modal         = Modal::new()?;

    let fields_clone        = fields.clone();
    let endpoint            = endpoint.to_string();
    let submit_button_clone = submit_button.clone();
    let modal_clone         = modal.clone();

    let closure = Closure::wrap(Box::new(move |e: Event| {
        e.prevent_default();

        let endpoint      = endpoint.clone();
        let fields        = fields_clone.clone();
        let modal         = modal_clone.clone();
        let submit_button = submit_button_clone.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let (form_data, debug_data) = process_form_data(&fields).await;
            
            log_form(&endpoint, &debug_data);
            

            submit_form_data(&endpoint, form_data, &modal, &submit_button).await;
        });
    }) as Box<dyn FnMut(_)>);

    form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())?;
    closure.forget();

    log("#### Fin script send form");
    Ok(())
}

async fn submit_form_data(endpoint: &str, form_data: FormData, modal: &Modal, submit_button: &HtmlButtonElement) {
    // Active le loader avant l'envoi
    set_button_loading_state(submit_button, true);
    delay_timeout(500).await;
    
    // Envoi de la requête
    match client_request::post_form(endpoint, &form_data).await {
        Ok(response) => {
            if response.is_success() {
                modal.show(&format!("✓ {}", response.get_message())).unwrap();
            } else {
                modal.show(&format!("⨯ Erreur: {}", response.get_message())).unwrap();
            }
        },
        Err(e) => {
            modal.show(&format!("⨯ Erreur: {:?}", e)).unwrap()
        },
    }

    // Désactive le loader une fois terminé
    set_button_loading_state(submit_button, false);
    submit_button.set_disabled(false);
}

fn create_form_fields(document: &Document, field_specs: Option<&[(&str, FieldType)]>) -> Result<Vec<FormField>, JsValue> {
    let mut fields = Vec::new();
    if let Some(specs) = field_specs {
        for &(field_id, ref field_type) in specs {
            let input = input(&document, field_id)?;
            if matches!(field_type, FieldType::File) {
                input.set_attribute("type", "file")?;
                input.set_attribute("multiple", "")?;
            }
            fields.push(FormField {
                id: field_id.to_string(),
                field_type: field_type.clone(),
                input,
            });
        }
    }
    Ok(fields)
}

fn find_submit_button(form: &Element, document: &Document, form_id: &str) -> Result<HtmlButtonElement, JsValue> {
    let button = document
        .query_selector(&format!("button[form='{}']", form_id))
        .map_err(|e| e)?
        .ok_or_else(|| JsValue::from_str("Submit button not found"))?;

    let button = button.dyn_into::<HtmlButtonElement>()
        .map_err(|e: Element| JsValue::from_str(&format!("Failed to convert to HtmlButtonElement: {:?}", e)))?;

    Ok(button)
}

fn set_button_loading_state(button: &HtmlButtonElement, is_loading: bool) {

    log(&format!("Setting button loading state: {}", is_loading));
    if is_loading {
        // Ajoute le loader
        let loader = format!("{}<div class='loader'></div>", button.inner_text());
        button.set_inner_html(&loader);
    } else {
        // Retire le loader en restaurant juste le texte
        button.set_inner_text(&button.inner_text().replace("Submit", "Submit"));
    }
}

async fn process_form_data(fields: &[FormField]) -> (FormData, Value) {
    let form_data = FormData::new().unwrap();
    let mut debug_data = json!({});

    if !fields.is_empty() {
        for field in fields.iter() {
            match field.field_type {
                FieldType::Text => {
                    let value = field.input.value();
                    form_data.append_with_str(&field.id, &value).unwrap();
                    debug_data[&field.id] = Value::String(value);
                }
                FieldType::File => {
                    if let Some(files) = field.input.files() {
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
        }
    }
    (form_data, debug_data)
}



fn input(document: &Document, element_name: &str) -> Result<HtmlInputElement, JsValue> {
    Ok(document
        .get_element_by_id(element_name)
        .ok_or_else(|| JsValue::from_str(&format!("Input element {} not found", element_name)))?
        .dyn_into::<HtmlInputElement>()?)
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