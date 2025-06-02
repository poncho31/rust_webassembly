use web_sys::{window, HtmlButtonElement, Event, Document};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use std::cell::RefCell;
use gloo_timers::future::TimeoutFuture;

use crate::{
    client_tools::log,
    client_request,
    modal::Modal,
    form::{FormField, FormConfig, FormProcessor, FieldType},
    validation::FormValidator
};

/// Main form handler that orchestrates form behavior
pub struct FormHandler {
    form_id: String,
    fields: Vec<FormField>,
    config: FormConfig,
    modal: Modal,
    submit_button: HtmlButtonElement,
    endpoint: String,
    validator: Option<FormValidator>,
}

impl FormHandler {
    /// Create a new form handler
    pub fn new(
        form_id: &str,
        endpoint: &str,
        field_specs: Option<&[(&str, FieldType)]>,
        config: FormConfig,
    ) -> Result<Self, JsValue> {
        let document = window()
            .ok_or_else(|| JsValue::from_str("Window not available"))?
            .document()
            .ok_or_else(|| JsValue::from_str("Document not available"))?;

        let form = document
            .get_element_by_id(form_id)
            .ok_or_else(|| JsValue::from_str(&format!("Form '{}' not found", form_id)))?;

        let fields = Self::create_form_fields(&document, field_specs)?;
        
        let submit_button = document
            .query_selector(&format!("button[form='{}']", form_id))?
            .or_else(|| form.query_selector("button[type='submit']").ok().flatten())
            .or_else(|| form.query_selector("input[type='submit']").ok().flatten())
            .ok_or_else(|| JsValue::from_str("Submit button not found"))?
            .dyn_into::<HtmlButtonElement>()?;

        let modal = Modal::new()?;        Ok(Self {
            form_id: form_id.to_string(),
            fields,
            config,
            modal,
            submit_button,
            endpoint: endpoint.to_string(),
            validator: None,
        })
    }

    /// Set a custom validator for the form
    pub fn with_validator(mut self, validator: FormValidator) -> Self {
        self.validator = Some(validator);
        self
    }

    /// Initialize the form with event listeners
    pub fn initialize(self) -> Result<(), JsValue> {
        let handler = Rc::new(RefCell::new(self));
        let handler_clone = Rc::clone(&handler);        let document = window().unwrap().document().unwrap();
        let form = document
            .get_element_by_id(&handler.borrow().form_id)
            .ok_or_else(|| JsValue::from_str(&format!("Form '{}' not found", handler.borrow().form_id)))?;

        let closure = Closure::wrap(Box::new(move |e: Event| {
            e.prevent_default();
            let handler_ref = handler_clone.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(err) = handler_ref.borrow().handle_submit().await {
                    log(&format!("Form submission error: {:?}", err));
                }
            });
        }) as Box<dyn FnMut(_)>);

        form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())?;
        closure.forget();

        Ok(())
    }

    /// Handle form submission
    async fn handle_submit(&self) -> Result<(), JsValue> {
        // Pre-submission validation
        if self.config.enable_validation {
            if let Err(errors) = self.validate_form() {
                self.show_validation_errors(&errors).await?;
                return Ok(());
            }
        }

        // Process form data
        let (form_data, debug_data) = FormProcessor::process_fields(&self.fields, &self.config).await?;
        
        // Log form data for debugging
        self.log_form_data(&debug_data);

        // Submit form with retry logic
        self.submit_with_retry(form_data).await
    }

    /// Validate the form using configured validator or built-in validation
    fn validate_form(&self) -> Result<(), Vec<String>> {
        if let Some(validator) = &self.validator {
            let form_values = FormProcessor::extract_values(&self.fields);
            let result = validator.validate(&form_values);
            
            if result.is_valid {
                Ok(())
            } else {
                let errors = result.errors.into_iter()
                    .map(|err| err.message)
                    .collect();
                Err(errors)
            }
        } else {
            // Use built-in field validation
            FormProcessor::validate_fields(&self.fields)
        }
    }

    /// Show validation errors to the user
    async fn show_validation_errors(&self, errors: &[String]) -> Result<(), JsValue> {
        let error_html = self.format_validation_errors(errors);
        self.modal.show(&error_html)?;

        // Focus on first error if configured
        if self.config.auto_focus_error {
            FormProcessor::focus_first_error(&self.fields)?;
        }

        Ok(())
    }

    /// Submit form data with retry logic
    async fn submit_with_retry(&self, form_data: web_sys::FormData) -> Result<(), JsValue> {
        let mut attempts = 0;
        let max_attempts = self.config.retry_attempts;

        while attempts < max_attempts {
            attempts += 1;

            // Show loading state
            if self.config.show_loading {
                self.set_loading_state(true)?;
            }

            // Attempt submission
            match client_request::post_form(&self.endpoint, &form_data).await {
                Ok(response) if response.is_success() => {
                    let message = self.config.success_message
                        .as_deref()
                        .unwrap_or("✓ Form submitted successfully!");
                    
                    self.modal.show(&format!("{} {}", message, response.get_message()))?;
                    
                    // Clear form on success if configured
                    // FormProcessor::clear_fields(&self.fields)?;
                    
                    if self.config.show_loading {
                        self.set_loading_state(false)?;
                    }
                    return Ok(());
                }
                Ok(response) => {
                    let error_msg = format!("⨯ Server Error: {}", response.get_message());
                    
                    if attempts >= max_attempts {
                        self.modal.show(&error_msg)?;                    } else {
                        log(&format!("Attempt {} failed, retrying...", attempts));
                        // Brief delay before retry
                        TimeoutFuture::new(1000).await;
                        continue;
                    }
                }
                Err(e) => {
                    let error_msg = self.config.error_message
                        .as_deref()
                        .map(|msg| format!("{}: {:?}", msg, e))
                        .unwrap_or_else(|| format!("⨯ Network Error: {:?}", e));
                    
                    if attempts >= max_attempts {
                        self.modal.show(&error_msg)?;                    } else {
                        log(&format!("Network error on attempt {}, retrying...", attempts));
                        TimeoutFuture::new(2000).await;
                        continue;
                    }
                }
            }

            if self.config.show_loading {
                self.set_loading_state(false)?;
            }
            break;
        }

        Ok(())
    }

    /// Set loading state on submit button
    fn set_loading_state(&self, loading: bool) -> Result<(), JsValue> {
        if loading {
            let current_text = self.submit_button.inner_text();
            self.submit_button.set_inner_html(&format!(
                "{}<div class='loader'></div>", 
                current_text
            ));
            self.submit_button.set_disabled(true);
        } else {
            let text = self.submit_button.inner_text();
            self.submit_button.set_inner_text(&text.replace("Submit", "Submit"));
            self.submit_button.set_disabled(false);
        }
        Ok(())
    }

    /// Create form fields from specifications
    fn create_form_fields(
        document: &Document,
        specs: Option<&[(&str, FieldType)]>,
    ) -> Result<Vec<FormField>, JsValue> {
        let mut fields = Vec::new();
        
        if let Some(specs) = specs {
            for &(id, ref field_type) in specs {
                let input = document
                    .get_element_by_id(id)
                    .ok_or_else(|| JsValue::from_str(&format!("Input '{}' not found", id)))?
                    .dyn_into::<web_sys::HtmlInputElement>()?;

                let required = input.has_attribute("required");
                let field = FormField::with_validation(
                    id.to_string(),
                    field_type.clone(),
                    input,
                    required,
                )?;

                fields.push(field);
            }
        }
        
        Ok(fields)
    }

    /// Format validation errors for display
    fn format_validation_errors(&self, errors: &[String]) -> String {
        let errors_html = errors.iter()
            .map(|error| format!("<li>{}</li>", error))
            .collect::<Vec<String>>()
            .join("");
        
        format!(
            "<div style='color: #cf222e;'><strong>Validation Errors:</strong><ul>{}</ul></div>", 
            errors_html
        )
    }

    /// Log form data for debugging
    fn log_form_data(&self, debug_data: &serde_json::Value) {
        log("=== Form Submission ===");
        log(&format!("Endpoint: {}", self.endpoint));
        log(&format!("Data: {}", debug_data.to_string()));
        log("======================");
    }
}

/// Convenience functions for backward compatibility
pub fn form_init(
    form_id: &str,
    endpoint: &str,
    field_specs: Option<&[(&str, FieldType)]>,
) -> Result<(), JsValue> {
    form_init_with_config(form_id, endpoint, field_specs, FormConfig::default())
}

pub fn form_init_with_config(
    form_id: &str,
    endpoint: &str,
    field_specs: Option<&[(&str, FieldType)]>,
    config: FormConfig,
) -> Result<(), JsValue> {
    let handler = FormHandler::new(form_id, endpoint, field_specs, config)?;
    handler.initialize()
}
