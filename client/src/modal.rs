use web_sys::{window, Document, Element, HtmlElement, Event};
use wasm_bindgen::prelude::*;

/// Modal structure representing a popup dialog
/// Contains the main element, content container and message area
#[derive(Clone)]
pub struct Modal {
    element: Element,    // Main modal container
    content: Element,    // Content wrapper
    message_container: Element,  // Message display area
}

impl Modal {
    /// Creates a new modal and adds it to the document
    pub fn new() -> Result<Self, JsValue> {
        let window = window().unwrap();
        let document = window.document().unwrap();
        
        // Create main modal container
        let modal = document.create_element("div")?;
        modal.set_class_name("modal");
        
        // Create content wrapper
        let content = document.create_element("div")?;
        content.set_class_name("modal-content");
        
        // Create message container
        let message_container = document.create_element("div")?;
        message_container.set_class_name("modal-message");
        
        // Create close button
        let close_btn = document.create_element("button")?;
        close_btn.set_class_name("modal-close");
        close_btn.set_inner_html("×");
        
        // Setup close button event handler
        let modal_ref = modal.clone();
        let close_callback = Closure::wrap(Box::new(move |e: Event| {
            e.prevent_default();
            modal_ref.set_class_name("modal");
        }) as Box<dyn FnMut(Event)>);
        
        // Setup outside click event handler
        let modal_ref = modal.clone();
        let outside_click = Closure::wrap(Box::new(move |e: Event| {
            let target = e.target().unwrap();
            let element: Element = target.dyn_into().unwrap();
            if element.class_name() == "modal show" {
                modal_ref.set_class_name("modal");
            }
        }) as Box<dyn FnMut(Event)>);
        
        // Add event listeners
        close_btn.add_event_listener_with_callback("click", close_callback.as_ref().unchecked_ref())?;
        modal.add_event_listener_with_callback("click", outside_click.as_ref().unchecked_ref())?;
        
        // Release callbacks
        close_callback.forget();
        outside_click.forget();
        
        // Construct modal hierarchy
        content.append_child(&close_btn)?;
        content.append_child(&message_container)?;
        modal.append_child(&content)?;
        document.body().unwrap().append_child(&modal)?;
        
        Ok(Modal {
            element: modal,
            content,
            message_container,
        })
    }

    /// Shows the modal with a message
    pub fn show(&self, message: &str) -> Result<(), JsValue> {
        self.message_container.set_inner_html(message);
        self.element.set_class_name("modal show");
        Ok(())
    }

    /// Hides the modal
    pub fn hide(&self) -> Result<(), JsValue> {
        self.element.set_class_name("modal");
        Ok(())
    }
}
