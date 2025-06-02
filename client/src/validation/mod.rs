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
    }    pub fn validate(&self, data: &HashMap<String, String>) -> ValidationResult {
        let mut errors = Vec::new();
        let empty_string = String::new();

        for (field, rule) in &self.rules {
            let value = data.get(field).unwrap_or(&empty_string);
            
            // Required validation
            if rule.required && value.trim().is_empty() {
                errors.push(ValidationError {
                    field: field.clone(),
                    message: rule.custom_message.clone()
                        .unwrap_or_else(|| format!("{} is required", field)),
                });
                continue;
            }

            // Skip further validation if field is empty and not required
            if value.trim().is_empty() {
                continue;
            }

            // Min length validation
            if let Some(min_len) = rule.min_length {
                if value.len() < min_len {
                    errors.push(ValidationError {
                        field: field.clone(),
                        message: format!("{} must be at least {} characters", field, min_len),
                    });
                }
            }

            // Max length validation
            if let Some(max_len) = rule.max_length {
                if value.len() > max_len {
                    errors.push(ValidationError {
                        field: field.clone(),
                        message: format!("{} must be no more than {} characters", field, max_len),
                    });
                }
            }

            // Pattern validation (basic email check for now)
            if let Some(pattern) = &rule.pattern {
                match pattern.as_str() {
                    "email" => {
                        if !is_valid_email(value) {
                            errors.push(ValidationError {
                                field: field.clone(),
                                message: format!("{} must be a valid email address", field),
                            });
                        }
                    },
                    "age" => {
                        if value.parse::<u32>().is_err() || value.parse::<u32>().unwrap() > 150 {
                            errors.push(ValidationError {
                                field: field.clone(),
                                message: format!("{} must be a valid age (0-150)", field),
                            });
                        }
                    },
                    _ => {}
                }
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        }
    }
}

fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

impl Default for FormValidator {
    fn default() -> Self {
        Self::new()            .add_rule("email", ValidationRule {
                required: true,
                min_length: Some(5),
                max_length: Some(255),
                pattern: Some("email".to_string()),
                custom_message: None,
                min_value: None,
                max_value: None,
            })
            .add_rule("firstname", ValidationRule {
                required: true,
                min_length: Some(2),
                max_length: Some(50),
                pattern: None,
                custom_message: None,
                min_value: None,
                max_value: None,
            })
            .add_rule("lastname", ValidationRule {
                required: true,
                min_length: Some(2),
                max_length: Some(50),
                pattern: None,
                custom_message: None,
                min_value: None,
                max_value: None,
            })
            .add_rule("age", ValidationRule {
                required: false,
                min_length: None,
                max_length: None,
                pattern: Some("age".to_string()),
                custom_message: None,
                min_value: None,
                max_value: None,
            })
    }
}

pub mod tests;
