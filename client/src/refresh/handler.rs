use crate::refresh::config::{RefreshConfig, ContentType};
use crate::client_request::fetch_json;
use crate::client_tools::log;
use web_sys::{HtmlElement, HtmlInputElement};
use wasm_bindgen::JsCast;
use serde_json::Value;

/// Gestionnaire pour les rafraîchissements automatiques
pub struct RefreshHandler {
    pub config: RefreshConfig,
}

impl RefreshHandler {
    /// Créer un nouveau gestionnaire
    pub fn new(config: RefreshConfig) -> Self {
        Self { config }
    }

    /// Exécuter un rafraîchissement
    pub async fn execute_refresh(&self) {
        log(&format!("🔄 Refreshing: {}", self.config.id));        // Utiliser client_request pour faire l'appel API
        match fetch_json::<Value>(&self.config.endpoint).await {
            Ok(response) => {
                if let Err(e) = self.update_dom(&response).await {
                    if self.config.show_errors {
                        self.show_error(&format!("DOM update error: {}", e));
                    }
                    log(&format!("❌ DOM update failed for {}: {}", self.config.id, e));
                }
            }
            Err(e) => {
                if self.config.show_errors {
                    self.show_error(&format!("API error: {:?}", e));
                }
                log(&format!("❌ Refresh failed for {}: {:?}", self.config.id, e));
            }
        }
    }

    /// Mettre à jour le DOM avec les données reçues
    async fn update_dom(&self, data: &Value) -> Result<(), String> {
        let window = web_sys::window().ok_or("No window")?;
        let document = window.document().ok_or("No document")?;
        
        let element = document
            .query_selector(&self.config.target_selector)
            .map_err(|_| "Query selector failed")?
            .ok_or_else(|| format!("Element not found: {}", self.config.target_selector))?;

        // Extraire la valeur des données JSON
        let value = if let Some(field) = &self.config.json_field {
            data.get(field)
                .ok_or_else(|| format!("Field '{}' not found in response", field))?
        } else {
            data
        };

        // Convertir en string et appliquer les transformations
        let content = self.process_value(value)?;

        // Mettre à jour l'élément selon le type de contenu
        match &self.config.content_type {
            ContentType::Text => {
                element.set_text_content(Some(&content));
            }
            ContentType::Html => {
                let html_element = element
                    .dyn_into::<HtmlElement>()
                    .map_err(|_| "Element is not an HTML element")?;
                html_element.set_inner_html(&content);
            }
            ContentType::Value => {
                let input_element = element
                    .dyn_into::<HtmlInputElement>()
                    .map_err(|_| "Element is not an input element")?;
                input_element.set_value(&content);
            }
            ContentType::Attribute(attr_name) => {
                element
                    .set_attribute(attr_name, &content)
                    .map_err(|_| format!("Failed to set attribute '{}'", attr_name))?;
            }
        }

        log(&format!("✅ Updated {}: {}", self.config.target_selector, content));
        Ok(())
    }

    /// Traiter et transformer une valeur JSON
    fn process_value(&self, value: &Value) -> Result<String, String> {
        let mut content = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => {
                if let Some(transform) = &self.config.transform {
                    if let Some(format) = &transform.format {
                        if format == "number" {
                            return Ok(self.apply_transform(&n.to_string()));
                        }
                    }
                }
                n.to_string()
            }
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => value.to_string(),
        };

        // Appliquer les transformations si définies
        if self.config.transform.is_some() {
            content = self.apply_transform(&content);
        }

        Ok(content)
    }

    /// Appliquer les transformations (préfixe, suffixe, format)
    fn apply_transform(&self, content: &str) -> String {
        if let Some(transform) = &self.config.transform {
            let mut result = content.to_string();
            
            if let Some(prefix) = &transform.prefix {
                result = format!("{}{}", prefix, result);
            }
            
            if let Some(suffix) = &transform.suffix {
                result = format!("{}{}", result, suffix);
            }
            
            result
        } else {
            content.to_string()
        }
    }

    /// Afficher une erreur dans l'interface
    fn show_error(&self, message: &str) {
        if let Ok(window) = web_sys::window().ok_or("No window") {
            if let Ok(document) = window.document().ok_or("No document") {
                if let Ok(Some(element)) = document.query_selector(&self.config.target_selector) {
                    match &self.config.content_type {
                        ContentType::Text => {
                            element.set_text_content(Some(&format!("Error: {}", message)));
                        }
                        ContentType::Html => {
                            if let Ok(html_element) = element.dyn_into::<HtmlElement>() {
                                html_element.set_inner_html(&format!("<span style='color: red;'>Error: {}</span>", message));
                            }
                        }
                        _ => {
                            // Pour les autres types, on affiche l'erreur en texte
                            element.set_text_content(Some("Error"));
                        }
                    }
                }
            }
        }
    }
}
