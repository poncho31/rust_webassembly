use web_sys::{HtmlInputElement, Element};
use serde::{Serialize, Deserialize};
use wasm_bindgen::{JsValue, JsCast};

/// Option for select fields, radio buttons, etc.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FieldOption {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

impl FieldOption {
    pub fn new<V: Into<String>, L: Into<String>>(value: V, label: L) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            selected: false,
        }
    }

    pub fn selected(mut self) -> Self {
        self.selected = true;
        self
    }
}

/// Field configuration with options and default values
#[derive(Clone, Debug)]
pub struct FieldConfig {
    pub field_type: FieldType,
    pub options: Option<Vec<FieldOption>>,
    pub default_value: Option<String>,
    pub placeholder: Option<String>,
    pub required: bool,
}

impl FieldConfig {
    pub fn new(field_type: FieldType) -> Self {
        Self {
            field_type,
            options: None,
            default_value: None,
            placeholder: None,
            required: false,
        }
    }

    pub fn with_options(mut self, options: Vec<FieldOption>) -> Self {
        self.options = Some(options);
        self
    }

    pub fn with_default_value<S: Into<String>>(mut self, value: S) -> Self {
        self.default_value = Some(value.into());
        self
    }

    pub fn with_placeholder<S: Into<String>>(mut self, placeholder: S) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Supported field types with validation patterns
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    Text,
    Email,
    Password,
    File,
    Date,
    Number,
    Tel,
    Url,
    TextArea,
    Select,
    Checkbox,
    Radio,
}

impl FieldType {
    /// Get the HTML input type for this field
    pub fn html_type(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Email => "email",
            Self::Password => "password",
            Self::File => "file",
            Self::Date => "date",
            Self::Number => "number",
            Self::Tel => "tel",
            Self::Url => "url",
            Self::TextArea => "textarea",
            Self::Select => "select",
            Self::Checkbox => "checkbox",
            Self::Radio => "radio",
        }
    }

    /// Check if this field type supports files
    pub fn supports_files(&self) -> bool {
        matches!(self, Self::File)
    }

    /// Check if this field type supports multiple values
    pub fn supports_multiple(&self) -> bool {
        matches!(self, Self::File | Self::Checkbox)
    }
}

/// Represents a form field with its metadata and DOM element
#[derive(Clone)]
pub struct FormField {
    id: String,
    field_type: FieldType,
    element: Element,  // Changed from HtmlInputElement to Element
    required: bool,
    label: Option<String>,
    config: Option<FieldConfig>,
}

impl FormField {
    pub fn new(
        id: String,
        field_type: FieldType,
        element: Element,
    ) -> Self {
        Self {
            id,
            field_type,
            element,
            required: false,
            label: None,
            config: None,
        }
    }

    /// Create a FormField with configuration
    pub fn with_config(
        id: String,
        config: FieldConfig,
        element: Element,
    ) -> Result<Self, JsValue> {
        // Set HTML attributes based on field type
        if config.field_type != FieldType::Select {
            element.set_attribute("type", config.field_type.html_type())?;
        }
        
        if config.field_type.supports_multiple() {
            element.set_attribute("multiple", "")?;
        }

        if config.required {
            element.set_attribute("required", "")?;
        }        // Set placeholder if provided (for input elements)
        if let Some(placeholder) = &config.placeholder {
            element.set_attribute("placeholder", placeholder)?;
        }

        // Set default value if provided
        if let Some(default_value) = &config.default_value {
            js_sys::Reflect::set(&element, &JsValue::from_str("value"), &JsValue::from_str(default_value))?;
        }

        // Handle select fields with options
        if config.field_type == FieldType::Select {
            if let Some(options) = &config.options {
                Self::populate_select_options(&element, options)?;
            }
        }

        Ok(Self {
            id,
            field_type: config.field_type.clone(),
            element,
            required: config.required,
            label: None,
            config: Some(config),
        })
    }    /// Populate select field with options
    fn populate_select_options(element: &Element, options: &[FieldOption]) -> Result<(), JsValue> {
        // Clear existing options
        element.set_inner_html("");

        // Add new options
        for option in options {
            let option_element: Element = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("option")?;
                
            option_element.set_attribute("value", &option.value)?;
            option_element.set_text_content(Some(&option.label));
            
            if option.selected {
                option_element.set_attribute("selected", "selected")?;
            }
            
            element.append_child(&option_element)?;
        }

        Ok(())
    }    /// Create a FormField with validation setup
    pub fn with_validation(
        id: String,
        field_type: FieldType,
        element: Element,
        required: bool,
    ) -> Result<Self, JsValue> {
        // Set HTML attributes based on field type
        if field_type != FieldType::Select && field_type != FieldType::TextArea {
            element.set_attribute("type", field_type.html_type())?;
        }
        
        if field_type.supports_multiple() {
            element.set_attribute("multiple", "")?;
        }

        if required {
            element.set_attribute("required", "")?;
        }

        Ok(Self {
            id,
            field_type,
            element,
            required,
            label: None,
            config: None,
        })
    }pub fn with_label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    // Getters
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn field_type(&self) -> &FieldType {
        &self.field_type
    }    pub fn element(&self) -> &Element {
        &self.element
    }    /// Get the element as HtmlInputElement if it is one
    pub fn input(&self) -> Option<HtmlInputElement> {
        self.element.dyn_ref::<HtmlInputElement>().cloned()
    }

    pub fn is_required(&self) -> bool {
        self.required
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn config(&self) -> Option<&FieldConfig> {
        self.config.as_ref()
    }    /// Get the current value of the field
    pub fn value(&self) -> String {
        // Try to get value property using reflection
        if let Ok(value) = js_sys::Reflect::get(&self.element, &JsValue::from_str("value")) {
            if let Some(value_str) = value.as_string() {
                return value_str;
            }
        }
        String::new()
    }

    /// Set the value of the field
    pub fn set_value(&self, value: &str) -> Result<(), JsValue> {
        js_sys::Reflect::set(&self.element, &JsValue::from_str("value"), &JsValue::from_str(value))?;
        Ok(())
    }

    /// Focus on this field
    pub fn focus(&self) -> Result<(), JsValue> {
        // Try to call focus method using reflection
        if let Ok(focus_fn) = js_sys::Reflect::get(&self.element, &JsValue::from_str("focus")) {
            if let Ok(focus_fn) = focus_fn.dyn_into::<js_sys::Function>() {
                focus_fn.call0(&self.element)?;
            }
        }
        Ok(())
    }/// Check if the field has files (for file inputs)
    pub fn has_files(&self) -> bool {
        self.field_type.supports_files() && 
        self.input().map_or(false, |input| input.files().map_or(false, |files| files.length() > 0))
    }

    /// Get files from the field (for file inputs)
    pub fn files(&self) -> Option<web_sys::FileList> {
        if self.field_type.supports_files() {
            self.input().and_then(|input| input.files())
        } else {
            None
        }
    }

    /// Validate the field's current value
    pub fn is_valid(&self) -> bool {
        if self.required && self.value().trim().is_empty() {
            return false;
        }

        match &self.field_type {
            FieldType::Email => {
                let value = self.value();
                !value.is_empty() && value.contains('@') && value.contains('.')
            }
            FieldType::Number => {
                let value = self.value();
                value.is_empty() || value.parse::<f64>().is_ok()
            }
            FieldType::Url => {
                let value = self.value();
                value.is_empty() || value.starts_with("http")
            }
            _ => true,
        }
    }

    /// Get validation error message if field is invalid
    pub fn validation_error(&self) -> Option<String> {
        if !self.is_valid() {
            match &self.field_type {
                FieldType::Email => Some(format!("{} must be a valid email address", 
                    self.label.as_deref().unwrap_or(&self.id))),
                FieldType::Number => Some(format!("{} must be a valid number", 
                    self.label.as_deref().unwrap_or(&self.id))),
                FieldType::Url => Some(format!("{} must be a valid URL", 
                    self.label.as_deref().unwrap_or(&self.id))),
                _ if self.required => Some(format!("{} is required", 
                    self.label.as_deref().unwrap_or(&self.id))),
                _ => None,
            }
        } else {
            None
        }
    }
}
