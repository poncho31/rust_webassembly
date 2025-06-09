use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use web_sys::{HtmlInputElement, HtmlElement};
use wasm_bindgen::{JsValue, JsCast};

/// Cache for DOM elements and form data to improve performance
pub struct FormCache {
    elements: RefCell<HashMap<String, HtmlElement>>,
    inputs: RefCell<HashMap<String, HtmlInputElement>>,
    form_data: RefCell<HashMap<String, String>>,
}

impl FormCache {
    pub fn new() -> Self {
        Self {
            elements: RefCell::new(HashMap::new()),
            inputs: RefCell::new(HashMap::new()),
            form_data: RefCell::new(HashMap::new()),
        }
    }

    /// Cache a DOM element by ID
    pub fn cache_element(&self, id: &str, element: HtmlElement) {
        self.elements.borrow_mut().insert(id.to_string(), element);
    }

    /// Get a cached DOM element
    pub fn get_element(&self, id: &str) -> Option<HtmlElement> {
        self.elements.borrow().get(id).cloned()
    }

    /// Cache an input element by ID
    pub fn cache_input(&self, id: &str, input: HtmlInputElement) {
        self.inputs.borrow_mut().insert(id.to_string(), input);
    }

    /// Get a cached input element
    pub fn get_input(&self, id: &str) -> Option<HtmlInputElement> {
        self.inputs.borrow().get(id).cloned()
    }

    /// Cache form field value
    pub fn cache_value(&self, field: &str, value: String) {
        self.form_data.borrow_mut().insert(field.to_string(), value);
    }

    /// Get cached form field value
    pub fn get_value(&self, field: &str) -> Option<String> {
        self.form_data.borrow().get(field).cloned()
    }

    /// Update cached value if it exists
    pub fn update_value(&self, field: &str, value: String) -> bool {
        if self.form_data.borrow().contains_key(field) {
            self.cache_value(field, value);
            true
        } else {
            false
        }
    }

    /// Clear all cached data
    pub fn clear(&self) {
        self.elements.borrow_mut().clear();
        self.inputs.borrow_mut().clear();
        self.form_data.borrow_mut().clear();
    }

    /// Clear only form data (keep DOM elements cached)
    pub fn clear_data(&self) {
        self.form_data.borrow_mut().clear();
    }

    /// Get all cached form data
    pub fn get_all_data(&self) -> HashMap<String, String> {
        self.form_data.borrow().clone()
    }

    /// Check if element is cached
    pub fn has_element(&self, id: &str) -> bool {
        self.elements.borrow().contains_key(id)
    }

    /// Check if input is cached
    pub fn has_input(&self, id: &str) -> bool {
        self.inputs.borrow().contains_key(id)
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            elements_count: self.elements.borrow().len(),
            inputs_count: self.inputs.borrow().len(),
            data_count: self.form_data.borrow().len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub elements_count: usize,
    pub inputs_count: usize,
    pub data_count: usize,
}

impl Default for FormCache {
    fn default() -> Self {
        Self::new()
    }
}

// Global cache instance (singleton pattern)
thread_local! {
    static FORM_CACHE: Rc<FormCache> = Rc::new(FormCache::new());
}

/// Get the global form cache instance
pub fn get_form_cache() -> Rc<FormCache> {
    FORM_CACHE.with(|cache| cache.clone())
}

/// Utility functions for cache management
pub fn cache_element_by_id(id: &str) -> Result<HtmlElement, JsValue> {
    let cache = get_form_cache();
    
    if let Some(element) = cache.get_element(id) {
        return Ok(element);
    }

    // Element not in cache, fetch it
    let window = web_sys::window().ok_or("Window not available")?;
    let document = window.document().ok_or("Document not available")?;        let element = document
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str(&format!("Element '{}' not found", id)))?
            .dyn_into::<HtmlElement>()?;

    cache.cache_element(id, element.clone());
    Ok(element)
}

pub fn cache_input_by_id(id: &str) -> Result<HtmlInputElement, JsValue> {
    let cache = get_form_cache();
    
    if let Some(input) = cache.get_input(id) {
        return Ok(input);
    }

    // Input not in cache, fetch it
    let window = web_sys::window().ok_or("Window not available")?;
    let document = window.document().ok_or("Document not available")?;        let input = document
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str(&format!("Input '{}' not found", id)))?
            .dyn_into::<HtmlInputElement>()?;

    cache.cache_input(id, input.clone());
    Ok(input)
}

/// Performance monitoring
pub struct PerformanceMonitor {
    start_time: f64,
    operations: RefCell<Vec<PerformanceEntry>>,
}

#[derive(Debug, Clone)]
pub struct PerformanceEntry {
    pub operation: String,
    pub duration_ms: f64,
    pub timestamp: f64,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        let start_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        Self {
            start_time,
            operations: RefCell::new(Vec::new()),
        }
    }

    pub fn start_operation(&self, name: &str) -> OperationTimer {
        OperationTimer::new(name.to_string(), self)
    }

    pub fn record_operation(&self, operation: String, duration_ms: f64) {
        let timestamp = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        self.operations.borrow_mut().push(PerformanceEntry {
            operation,
            duration_ms,
            timestamp,
        });
    }

    pub fn get_average_duration(&self, operation: &str) -> Option<f64> {
        let operations = self.operations.borrow();
        let matching: Vec<&PerformanceEntry> = operations
            .iter()
            .filter(|entry| entry.operation == operation)
            .collect();

        if matching.is_empty() {
            None
        } else {
            let total: f64 = matching.iter().map(|e| e.duration_ms).sum();
            Some(total / matching.len() as f64)
        }
    }

    pub fn get_all_operations(&self) -> Vec<PerformanceEntry> {
        self.operations.borrow().clone()
    }
}

pub struct OperationTimer<'a> {
    name: String,
    start_time: f64,
    monitor: &'a PerformanceMonitor,
}

impl<'a> OperationTimer<'a> {
    fn new(name: String, monitor: &'a PerformanceMonitor) -> Self {
        let start_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        Self {
            name,
            start_time,
            monitor,
        }
    }
}

impl<'a> Drop for OperationTimer<'a> {
    fn drop(&mut self) {
        let end_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        let duration = end_time - self.start_time;
        self.monitor.record_operation(self.name.clone(), duration);
    }
}
