use wasm_bindgen::prelude::*;
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
            pattern: Some("number".to_string()),
            ..Default::default()
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

pub struct FormValidator {
    rules: HashMap<String, ValidationRule>,
}

impl FormValidator {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn add_rule(mut self, field: &str, rule: ValidationRule) -> Self {
        self.rules.insert(field.to_string(), rule);
        self
    }

    pub fn validate(&self, data: &HashMap<String, String>) -> ValidationResult {
        let mut errors = Vec::new();
        let empty_string = String::new();

        for (field, rule) in &self.rules {
            let value = data.get(field).unwrap_or(&empty_string);
            
            if let Some(error) = self.validate_field(field, value, rule) {
                errors.push(error);
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        }
    }

    fn validate_field(&self, field: &str, value: &str, rule: &ValidationRule) -> Option<ValidationError> {
        // Required validation
        if rule.required && value.trim().is_empty() {
            return Some(ValidationError {
                field: field.to_string(),
                message: rule.custom_message.clone()
                    .unwrap_or_else(|| format!("{} is required", field)),
            });
        }

        // Skip further validation if field is empty and not required
        if value.trim().is_empty() {
            return None;
        }

        // Length validations
        if let Some(min_len) = rule.min_length {
            if value.len() < min_len {
                return Some(ValidationError {
                    field: field.to_string(),
                    message: format!("{} must be at least {} characters", field, min_len),
                });
            }
        }

        if let Some(max_len) = rule.max_length {
            if value.len() > max_len {
                return Some(ValidationError {
                    field: field.to_string(),
                    message: format!("{} must be no more than {} characters", field, max_len),
                });
            }
        }

        // Pattern validation
        if let Some(pattern) = &rule.pattern {
            if let Some(error) = self.validate_pattern(field, value, pattern) {
                return Some(error);
            }
        }

        // Numeric value validation
        if let Some(min_val) = rule.min_value {
            if let Ok(num_value) = value.parse::<f64>() {
                if num_value < min_val {
                    return Some(ValidationError {
                        field: field.to_string(),
                        message: format!("{} must be at least {}", field, min_val),
                    });
                }
            }
        }

        if let Some(max_val) = rule.max_value {
            if let Ok(num_value) = value.parse::<f64>() {
                if num_value > max_val {
                    return Some(ValidationError {
                        field: field.to_string(),
                        message: format!("{} must be at most {}", field, max_val),
                    });
                }
            }
        }

        None
    }

    fn validate_pattern(&self, field: &str, value: &str, pattern: &str) -> Option<ValidationError> {
        match pattern {
            "email" => {
                if !is_valid_email(value) {
                    Some(ValidationError {
                        field: field.to_string(),
                        message: format!("{} must be a valid email address", field),
                    })
                } else {
                    None
                }
            },
            "number" => {
                if value.parse::<f64>().is_err() {
                    Some(ValidationError {
                        field: field.to_string(),
                        message: format!("{} must be a valid number", field),
                    })
                } else {
                    None
                }
            },
            "age" => {
                match value.parse::<u32>() {
                    Ok(age) if age > 150 => Some(ValidationError {
                        field: field.to_string(),
                        message: format!("{} must be a valid age (0-150)", field),
                    }),
                    Err(_) => Some(ValidationError {
                        field: field.to_string(),
                        message: format!("{} must be a valid age", field),
                    }),
                    _ => None
                }
            },
            "phone" => {
                // Basic phone validation (digits, spaces, dashes, plus)
                let phone_chars: String = value.chars()
                    .filter(|c| c.is_ascii_digit() || *c == ' ' || *c == '-' || *c == '+' || *c == '(' || *c == ')')
                    .collect();
                
                if phone_chars.len() < 10 {
                    Some(ValidationError {
                        field: field.to_string(),
                        message: format!("{} must be a valid phone number", field),
                    })
                } else {
                    None
                }
            },
            "url" => {
                if !value.starts_with("http://") && !value.starts_with("https://") {
                    Some(ValidationError {
                        field: field.to_string(),
                        message: format!("{} must be a valid URL", field),
                    })
                } else {
                    None
                }
            },
            _ => None
        }
    }
}

/// Improved email validation
fn is_valid_email(email: &str) -> bool {
    // More comprehensive email validation
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    
    let local = parts[0];
    let domain = parts[1];
    
    // Basic checks
    !local.is_empty() && 
    !domain.is_empty() && 
    domain.contains('.') && 
    email.len() >= 5 && 
    email.len() <= 255 &&
    !email.starts_with('.') && 
    !email.ends_with('.') &&
    !email.contains("..") &&
    local.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '+') &&
    domain.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-')
}

impl Default for FormValidator {
    fn default() -> Self {
        Self::new()
            .add_rule("email", ValidationRule::email())
            .add_rule("firstname", ValidationRule::text(2, 50))
            .add_rule("lastname", ValidationRule::text(2, 50))
            .add_rule("age", ValidationRule {
                required: false,
                pattern: Some("age".to_string()),
                ..Default::default()
            })
    }
}
