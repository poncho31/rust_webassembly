use web_sys::{window, HtmlInputElement, Event, Document, Element, Window, FormData};
use wasm_bindgen::prelude::*;
use crate::client_tools::log;
use crate::client_request;
use serde_json::{Value, json};

#[derive(Clone)]
pub enum FieldType {
    Text,
    File,
}

/// Structure représentant un champ de formulaire
#[derive(Clone)]
pub struct FormField {
    id: String,
    field_type: FieldType,
    input: HtmlInputElement,
}

pub fn form_init(form_id: &str, endpoint: &str, field_specs: &[(&str, FieldType)]) -> Result<(), JsValue> {
    log("#### Debut script send form");

    let window  : Window   = window().unwrap();
    let document: Document = window.document().unwrap();
    let form    : Element  = document.get_element_by_id(form_id).ok_or_else(|| JsValue::from_str("Form not found"))?;

    // Créer le mappage des champs de formulaire
    let mut fields: Vec<FormField> = Vec::new();
    for &(field_id, ref field_type) in field_specs {
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

    let fields_clone = fields.clone();
    let endpoint = endpoint.to_string();
    // Fonction asynchrone pour gérer la soumission du formulaire
    let closure = Closure::wrap(Box::new(move |e: Event| {
        e.prevent_default();
        
        let form_data = FormData::new().unwrap();
        let mut debug_data = json!({});  // Pour le logging

        for field in fields_clone.iter() {
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

        // Log détaillé des données du formulaire
        log("=== Form Data ===");
        log(&format!("Endpoint: {}", endpoint));
        log(&format!("Content: {}", debug_data.to_string()));
        log("===============");

        let _ = client_request::post_form(&endpoint, &form_data);
    }) as Box<dyn FnMut(_)>);

    form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())?;
    closure.forget();

    log("#### Fin script send form");
    Ok(())
}

fn input(document: &Document, element_name: &str) -> Result<HtmlInputElement, JsValue> {
    Ok(document
        .get_element_by_id(element_name)
        .ok_or_else(|| JsValue::from_str(&format!("Input element {} not found", element_name)))?
        .dyn_into::<HtmlInputElement>()?)
}