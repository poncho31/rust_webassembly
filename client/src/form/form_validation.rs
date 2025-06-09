//! Form validation module - consolidated validation system
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub required: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub custom_message: Option<String>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

impl Default for ValidationRule {
    fn default() -> Self {
        Self {
            required: false,
            min_length: None,
            max_length: None,
            pattern: None,
            custom_message: None,
            min_value: None,
            max_value: None,
        }
    }
}

impl ValidationRule {
    pub fn required() -> Self {
        Self {
            required: true,
            ..Default::default()
        }
    }

    pub fn optional() -> Self {
        Self {
            required: false,
            ..Default::default()
        }
    }

    pub fn email() -> Self {
        Self {
            required: true,
            min_length: Some(5),
            max_length: Some(255),
            pattern: Some("email".to_string()),
            ..Default::default()
        }
    }

    pub fn text(min_len: usize, max_len: usize) -> Self {
        Self {
            required: true,
            min_length: Some(min_len),
            max_length: Some(max_len),
            ..Default::default()
        }
    }

    pub fn number(min_val: f64, max_val: f64) -> Self {
        Self {
            required: true,
            min_value: Some(min_val),
            max_value: Some(max_val),
            ..Default::default()
        }
    }

    pub fn with_message<S: Into<String>>(mut self, message: S) -> Self {
        self.custom_message = Some(message.into());
        self
    }

    pub fn validate(&self, value: &str, field_name: &str) -> Result<(), String> {
        // Required field validation
        if self.required && value.trim().is_empty() {
            return Err(self.custom_message.clone()
                .unwrap_or_else(|| format!("{} is required", field_name)));
        }

        // Skip other validations if field is empty and not required
        if value.trim().is_empty() && !self.required {
            return Ok(());
        }

        // Length validations
        if let Some(min_len) = self.min_length {
            if value.len() < min_len {
                return Err(self.custom_message.clone()
                    .unwrap_or_else(|| format!("{} must be at least {} characters", field_name, min_len)));
            }
        }

        if let Some(max_len) = self.max_length {
            if value.len() > max_len {
                return Err(self.custom_message.clone()
                    .unwrap_or_else(|| format!("{} must be at most {} characters", field_name, max_len)));
            }
        }

        // Pattern validation
        if let Some(pattern) = &self.pattern {
            match pattern.as_str() {
                "email" => {
                    if !is_valid_email(value) {
                        return Err(self.custom_message.clone()
                            .unwrap_or_else(|| format!("{} must be a valid email address", field_name)));
                    }
                }
                "url" => {
                    if !is_valid_url(value) {
                        return Err(self.custom_message.clone()
                            .unwrap_or_else(|| format!("{} must be a valid URL", field_name)));
                    }
                }
                _ => {
                    // Custom regex patterns could be added here
                }
            }
        }

        // Number validations
        if let (Some(min_val), Some(max_val)) = (self.min_value, self.max_value) {
            if let Ok(num_val) = value.parse::<f64>() {
                if num_val < min_val || num_val > max_val {
                    return Err(self.custom_message.clone()
                        .unwrap_or_else(|| format!("{} must be between {} and {}", field_name, min_val, max_val)));
                }
            } else {
                return Err(self.custom_message.clone()
                    .unwrap_or_else(|| format!("{} must be a valid number", field_name)));
            }
        }

        Ok(())
    }
}

/// Form validator that manages multiple validation rules
#[derive(Debug, Clone)]
pub struct FormValidator {
    rules: HashMap<String, ValidationRule>,
}

impl FormValidator {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn add_rule<S: Into<String>>(mut self, field: S, rule: ValidationRule) -> Self {
        self.rules.insert(field.into(), rule);
        self
    }

    pub fn validate(&self, data: &HashMap<String, String>) -> ValidationResult {
        let mut errors = Vec::new();

        for (field, rule) in &self.rules {
            let value = data.get(field).map(|s| s.as_str()).unwrap_or("");
            if let Err(error_message) = rule.validate(value, field) {
                errors.push(ValidationError {
                    field: field.clone(),
                    message: error_message,
                });
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        }
    }

    pub fn validate_field(&self, field: &str, value: &str) -> Result<(), String> {
        if let Some(rule) = self.rules.get(field) {
            rule.validate(value, field)
        } else {
            Ok(())
        }
    }

    pub fn has_rule(&self, field: &str) -> bool {
        self.rules.contains_key(field)
    }
}

impl Default for FormValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

/// Individual validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Simple email validation
fn is_valid_email(email: &str) -> bool {
    email.contains('@') && 
    email.contains('.') && 
    email.len() >= 5 &&
    !email.starts_with('@') &&
    !email.ends_with('@') &&
    !email.starts_with('.') &&
    !email.ends_with('.')
}

/// Simple URL validation
fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || 
    url.starts_with("https://") || 
    url.starts_with("ftp://")
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let rule = ValidationRule::email();
        assert!(rule.validate("test@example.com", "email").is_ok());
        assert!(rule.validate("invalid", "email").is_err());
        assert!(rule.validate("", "email").is_err());
    }

    #[test]
    fn test_text_validation() {
        let rule = ValidationRule::text(3, 10);
        assert!(rule.validate("hello", "text").is_ok());
        assert!(rule.validate("hi", "text").is_err()); // too short
        assert!(rule.validate("this is too long", "text").is_err()); // too long
    }

    #[test]
    fn test_number_validation() {
        let rule = ValidationRule::number(18.0, 65.0);
        assert!(rule.validate("25", "age").is_ok());
        assert!(rule.validate("15", "age").is_err()); // too low
        assert!(rule.validate("70", "age").is_err()); // too high
        assert!(rule.validate("abc", "age").is_err()); // not a number
    }

    #[test]
    fn test_form_validator() {
        let mut data = HashMap::new();
        data.insert("email".to_string(), "test@example.com".to_string());
        data.insert("age".to_string(), "25".to_string());

        let validator = FormValidator::new()
            .add_rule("email", ValidationRule::email())
            .add_rule("age", ValidationRule::number(18.0, 65.0));

        let result = validator.validate(&data);
        assert!(result.is_valid);

        data.insert("email".to_string(), "invalid".to_string());
        let result = validator.validate(&data);
        assert!(!result.is_valid);
    }
}
