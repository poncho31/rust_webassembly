#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_validation_rule_builders() {
        let email_rule = ValidationRule::email();
        assert!(email_rule.required);
        assert_eq!(email_rule.min_length, Some(5));
        assert_eq!(email_rule.max_length, Some(255));
        assert_eq!(email_rule.pattern, Some("email".to_string()));

        let text_rule = ValidationRule::text(2, 50);
        assert!(text_rule.required);
        assert_eq!(text_rule.min_length, Some(2));
        assert_eq!(text_rule.max_length, Some(50));

        let number_rule = ValidationRule::number(0.0, 100.0);
        assert!(number_rule.required);
        assert_eq!(number_rule.min_value, Some(0.0));
        assert_eq!(number_rule.max_value, Some(100.0));
    }

    #[test]
    fn test_email_validation() {
        // Valid emails
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@domain.co.uk"));
        assert!(is_valid_email("test+tag@example.org"));
        
        // Invalid emails
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("test@"));
        assert!(!is_valid_email("test..test@example.com"));
        assert!(!is_valid_email(".test@example.com"));
        assert!(!is_valid_email("test@example."));
    }

    #[test]
    fn test_form_validator() {
        let validator = FormValidator::new()
            .add_rule("email", ValidationRule::email())
            .add_rule("name", ValidationRule::text(2, 50));

        let mut data = HashMap::new();
        data.insert("email".to_string(), "test@example.com".to_string());
        data.insert("name".to_string(), "John Doe".to_string());

        let result = validator.validate(&data);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validation_errors() {
        let validator = FormValidator::new()
            .add_rule("email", ValidationRule::email())
            .add_rule("name", ValidationRule::text(2, 50));

        let mut data = HashMap::new();
        data.insert("email".to_string(), "invalid-email".to_string());
        data.insert("name".to_string(), "A".to_string()); // Too short

        let result = validator.validate(&data);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 2);
        
        // Check error messages
        let email_error = result.errors.iter()
            .find(|e| e.field == "email")
            .unwrap();
        assert!(email_error.message.contains("valid email"));
        
        let name_error = result.errors.iter()
            .find(|e| e.field == "name")
            .unwrap();
        assert!(name_error.message.contains("at least 2 characters"));
    }

    #[test]
    fn test_required_field_validation() {
        let validator = FormValidator::new()
            .add_rule("required_field", ValidationRule::required());

        let mut data = HashMap::new();
        data.insert("required_field".to_string(), "".to_string());

        let result = validator.validate(&data);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("required"));
    }

    #[test]
    fn test_number_validation() {
        let validator = FormValidator::new()
            .add_rule("age", ValidationRule::number(0.0, 150.0));

        // Valid number
        let mut data = HashMap::new();
        data.insert("age".to_string(), "25".to_string());
        let result = validator.validate(&data);
        assert!(result.is_valid);

        // Too high
        data.insert("age".to_string(), "200".to_string());
        let result = validator.validate(&data);
        assert!(!result.is_valid);
        assert!(result.errors[0].message.contains("at most 150"));

        // Invalid number
        data.insert("age".to_string(), "not-a-number".to_string());
        let result = validator.validate(&data);
        assert!(!result.is_valid);
        assert!(result.errors[0].message.contains("valid number"));
    }

    #[test]
    fn test_phone_validation() {
        let validator = FormValidator::new()
            .add_rule("phone", ValidationRule {
                required: true,
                pattern: Some("phone".to_string()),
                ..Default::default()
            });

        // Valid phone numbers
        let valid_phones = vec![
            "+1-555-123-4567",
            "(555) 123-4567",
            "555.123.4567",
            "15551234567",
        ];

        for phone in valid_phones {
            let mut data = HashMap::new();
            data.insert("phone".to_string(), phone.to_string());
            let result = validator.validate(&data);
            assert!(result.is_valid, "Phone {} should be valid", phone);
        }

        // Invalid phone (too short)
        let mut data = HashMap::new();
        data.insert("phone".to_string(), "123".to_string());
        let result = validator.validate(&data);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_url_validation() {
        let validator = FormValidator::new()
            .add_rule("website", ValidationRule {
                required: true,
                pattern: Some("url".to_string()),
                ..Default::default()
            });

        // Valid URLs
        let mut data = HashMap::new();
        data.insert("website".to_string(), "https://example.com".to_string());
        let result = validator.validate(&data);
        assert!(result.is_valid);

        data.insert("website".to_string(), "http://test.org".to_string());
        let result = validator.validate(&data);
        assert!(result.is_valid);

        // Invalid URL
        data.insert("website".to_string(), "not-a-url".to_string());
        let result = validator.validate(&data);
        assert!(!result.is_valid);
        assert!(result.errors[0].message.contains("valid URL"));
    }

    #[test]
    fn test_default_validator() {
        let validator = FormValidator::default();

        let mut data = HashMap::new();
        data.insert("email".to_string(), "test@example.com".to_string());
        data.insert("firstname".to_string(), "John".to_string());
        data.insert("lastname".to_string(), "Doe".to_string());
        data.insert("age".to_string(), "25".to_string());

        let result = validator.validate(&data);
        assert!(result.is_valid);
    }
}
