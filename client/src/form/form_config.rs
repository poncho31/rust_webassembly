use serde::{Serialize, Deserialize};

/// Configuration options for form behavior
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormConfig {
    /// Enable client-side validation before submission
    pub enable_validation: bool,
    /// Show loading indicator during form submission
    pub show_loading: bool,
    /// Automatically focus on the first error field
    pub auto_focus_error: bool,
    /// Debounce delay for real-time validation (in milliseconds)
    pub debounce_ms: u32,
    /// Maximum file size allowed (in bytes)
    pub max_file_size: Option<u64>,
    /// Custom success message template
    pub success_message: Option<String>,
    /// Custom error message template
    pub error_message: Option<String>,
    /// Retry attempts for failed submissions
    pub retry_attempts: u8,
}

impl Default for FormConfig {
    fn default() -> Self {
        Self {
            enable_validation: true,
            show_loading: true,
            auto_focus_error: true,
            debounce_ms: 300,
            max_file_size: Some(5 * 1024 * 1024), // 5MB
            success_message: None,
            error_message: None,
            retry_attempts: 3,
        }
    }
}

impl FormConfig {
    pub fn builder() -> FormConfigBuilder {
        FormConfigBuilder::default()
    }
}

/// Builder pattern for FormConfig
#[derive(Default)]
pub struct FormConfigBuilder {
    config: FormConfig,
}

impl FormConfigBuilder {
    pub fn validation(mut self, enabled: bool) -> Self {
        self.config.enable_validation = enabled;
        self
    }

    pub fn loading(mut self, enabled: bool) -> Self {
        self.config.show_loading = enabled;
        self
    }

    pub fn auto_focus_error(mut self, enabled: bool) -> Self {
        self.config.auto_focus_error = enabled;
        self
    }

    pub fn debounce_ms(mut self, ms: u32) -> Self {
        self.config.debounce_ms = ms;
        self
    }

    pub fn max_file_size(mut self, size: u64) -> Self {
        self.config.max_file_size = Some(size);
        self
    }

    pub fn success_message<S: Into<String>>(mut self, message: S) -> Self {
        self.config.success_message = Some(message.into());
        self
    }

    pub fn error_message<S: Into<String>>(mut self, message: S) -> Self {
        self.config.error_message = Some(message.into());
        self
    }

    pub fn retry_attempts(mut self, attempts: u8) -> Self {
        self.config.retry_attempts = attempts;
        self
    }

    pub fn build(self) -> FormConfig {
        self.config
    }
}
