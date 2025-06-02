use web_sys::{window, HtmlInputElement, HtmlButtonElement, Event, Document, FormData};
use wasm_bindgen::prelude::*;
use crate::{client_tools::log, client_request, modal::Modal, validation::{FormValidator, ValidationResult}};
use serde_json::{Value, json};
use gloo_timers::callback::Timeout;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum FieldType { 
    Text, 
    Email, 
    File, 
    Date,
    Number,
}

#[derive(Clone)]
pub struct FormField {
    id: String,
    field_type: FieldType,
    input: HtmlInputElement,
}

#[derive(Clone)]
pub struct FormConfig {
    pub enable_validation: bool,
    pub show_loading: bool,
    pub auto_focus_error: bool,
    pub debounce_ms: u32,
}

impl Default for FormConfig {
    fn default() -> Self {
        Self {
            enable_validation: true,
            show_loading: true,
            auto_focus_error: true,
            debounce_ms: 300,
        }
    }
}

pub fn form_init(form_id: &str, endpoint: &str, field_specs: Option<&[(&str, FieldType)]>) -> Result<(), JsValue> {
    form_init_with_config(form_id, endpoint, field_specs, FormConfig::default())
}

pub fn form_init_with_config(
    form_id: &str, 
    endpoint: &str, 
    field_specs: Option<&[(&str, FieldType)]>,
    config: FormConfig
) -> Result<(), JsValue> {
    let document = window().unwrap().document().unwrap();
    let form = document.get_element_by_id(form_id)
        .ok_or_else(|| JsValue::from_str("Form not found"))?;

    let fields = create_form_fields(&document, field_specs)?;
      // Chercher le bouton spécifiquement lié à ce formulaire
    let submit_button = document.query_selector(&format!("button[form='{}'][type='submit']", form_id))?
        .or_else(|| form.query_selector("button[type='submit']").ok().flatten())
        .ok_or_else(|| JsValue::from_str(&format!("Submit button not found for form '{}'", form_id)))?
        .dyn_into::<HtmlButtonElement>()?;
    
    let modal = Modal::new()?;

    let closure = create_submit_handler(endpoint.to_string(), fields, modal, submit_button, config);
    form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}

fn create_submit_handler(
    endpoint: String, 
    fields: Vec<FormField>, 
    modal: Modal, 
    button: HtmlButtonElement,
    config: FormConfig
) -> Closure<dyn FnMut(Event)> {
    Closure::wrap(Box::new(move |e: Event| {
        e.prevent_default();
        e.stop_propagation(); // Empêche la propagation vers d'autres éléments
        
        let (endpoint, fields, modal, button, config) = (
            endpoint.clone(), 
            fields.clone(), 
            modal.clone(), 
            button.clone(),
            config.clone()
        );
        
        wasm_bindgen_futures::spawn_local(async move {
            // Validation avant soumission
            if config.enable_validation {
                let form_values = extract_form_values(&fields);
                let validator = FormValidator::default();
                let validation_result = validator.validate(&form_values);
                
                if !validation_result.is_valid {
                    let error_message = format_validation_errors(&validation_result);
                    modal.show(&error_message).unwrap();
                    
                    if config.auto_focus_error && !validation_result.errors.is_empty() {
                        if let Some(first_error) = validation_result.errors.first() {
                            focus_field(&fields, &first_error.field);
                        }
                    }
                    return;
                }
            }

            let (form_data, debug_data) = process_form_data(&fields).await;
            log_form(&endpoint, &debug_data);
            submit_form_data(&endpoint, form_data, &modal, &button, &config).await;
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

            // Configuration du type d'input selon le FieldType
            match field_type {
                FieldType::File => {
                    input.set_attribute("type", "file")?;
                    input.set_attribute("multiple", "")?;
                },
                FieldType::Email => {
                    input.set_attribute("type", "email")?;
                },
                FieldType::Date => {
                    input.set_attribute("type", "date")?;
                },
                FieldType::Number => {
                    input.set_attribute("type", "number")?;
                },
                FieldType::Text => {
                    input.set_attribute("type", "text")?;
                },
            }

            fields.push(FormField { id: id.to_string(), field_type: field_type.clone(), input });
        }
    }
    Ok(fields)
}

async fn submit_form_data(
    endpoint: &str, 
    form_data: FormData, 
    modal: &Modal, 
    button: &HtmlButtonElement,
    config: &FormConfig
) {
    if config.show_loading {
        button.set_inner_html(&format!("{}<div class='loader'></div>", button.inner_text()));
    }
    
    let result = match client_request::post_form(endpoint, &form_data).await {
        Ok(response) if response.is_success() => format!("✓ {}", response.get_message()),
        Ok(response) => format!("⨯ Erreur: {}", response.get_message()),
        Err(e) => format!("⨯ Erreur: {:?}", e),
    };

    modal.show(&result).unwrap();
    
    if config.show_loading {
        button.set_inner_text(&button.inner_text().replace("Submit", "Submit"));
    }
}

async fn process_form_data(fields: &[FormField]) -> (FormData, Value) {
    let form_data = FormData::new().unwrap();
    let mut debug_data = json!({});

    for field in fields {
        match field.field_type {
            FieldType::Text | FieldType::Email | FieldType::Date | FieldType::Number => {
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

// Nouvelles fonctions utilitaires
fn extract_form_values(fields: &[FormField]) -> HashMap<String, String> {
    let mut values = HashMap::new();
    for field in fields {
        if !matches!(field.field_type, FieldType::File) {
            values.insert(field.id.clone(), field.input.value());
        }
    }
    values
}

fn format_validation_errors(result: &ValidationResult) -> String {
    let errors_html = result.errors.iter()
        .map(|error| format!("<li>{}</li>", error.message))
        .collect::<Vec<String>>()
        .join("");
    
    format!("<div style='color: #cf222e;'><strong>Erreurs de validation:</strong><ul>{}</ul></div>", errors_html)
}

fn focus_field(fields: &[FormField], field_id: &str) {
    if let Some(field) = fields.iter().find(|f| f.id == field_id) {
        let _ = field.input.focus();
    }
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