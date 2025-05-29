use web_sys::{window, HtmlInputElement, Event, Document, Element, Window, File, FileList};
use wasm_bindgen::prelude::*;
use crate::client_tools::log;
use crate::wasm_bindgen;

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

pub fn form_init(form_id: &str, field_specs: &[(&str, FieldType)]) -> Result<(), JsValue> {
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
    // Fonction asynchrone pour gérer la soumission du formulaire
    let closure = Closure::wrap(Box::new(move |e: Event| {
        e.prevent_default();
        
        let mut log_parts = Vec::new();
        for field in fields_clone.iter() {
            match field.field_type {
                FieldType::Text => {
                    log_parts.push(format!("{}: {}", field.id, field.input.value()));
                }
                FieldType::File => {
                    if let Some(files) = field.input.files() {
                        let mut files_info = Vec::new();
                        let length = files.length();
                        for i in 0..length {
                            if let Some(file) = files.item(i) {
                                files_info.push(format!("{}({} bytes)", 
                                    file.name(),
                                    file.size()
                                ));
                            }
                        }
                        if !files_info.is_empty() {
                            log_parts.push(format!("{}: [{}]", 
                                field.id,
                                files_info.join(", ")
                            ));
                        }
                    }
                }
            }
        }
        log(&format!("Form submitted with {}", log_parts.join(", ")));
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