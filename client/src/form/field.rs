use web_sys::HtmlInputElement;
use serde::{Serialize, Deserialize};
use wasm_bindgen::JsValue;

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
    input: HtmlInputElement,
    required: bool,
    label: Option<String>,
}

impl FormField {
    pub fn new(
        id: String,
        field_type: FieldType,
        input: HtmlInputElement,
    ) -> Self {
        Self {
            id,
            field_type,
            input,
            required: false,
            label: None,
        }
    }

    /// Create a FormField with validation setup
    pub fn with_validation(
        id: String,
        field_type: FieldType,
        input: HtmlInputElement,
        required: bool,
    ) -> Result<Self, JsValue> {
        // Set HTML attributes based on field type
        input.set_attribute("type", field_type.html_type())?;
        
        if field_type.supports_multiple() {
            input.set_attribute("multiple", "")?;
        }

        if required {
            input.set_attribute("required", "")?;
        }

        Ok(Self {
            id,
            field_type,
            input,
            required,
            label: None,
        })
    }

    pub fn with_label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    // Getters
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn field_type(&self) -> &FieldType {
        &self.field_type
    }

    pub fn input(&self) -> &HtmlInputElement {
        &self.input
    }

    pub fn is_required(&self) -> bool {
        self.required
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Get the current value of the field
    pub fn value(&self) -> String {
        self.input.value()
    }

    /// Set the value of the field
    pub fn set_value(&self, value: &str) -> Result<(), JsValue> {
        self.input.set_value(value);
        Ok(())
    }

    /// Focus on this field
    pub fn focus(&self) -> Result<(), JsValue> {
        self.input.focus()
    }

    /// Check if the field has files (for file inputs)
    pub fn has_files(&self) -> bool {
        self.field_type.supports_files() && 
        self.input.files().map_or(false, |files| files.length() > 0)
    }

    /// Get files from the field (for file inputs)
    pub fn files(&self) -> Option<web_sys::FileList> {
        if self.field_type.supports_files() {
            self.input.files()
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
