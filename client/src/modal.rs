use web_sys::{window, Document, Element, HtmlElement, Event};
use wasm_bindgen::prelude::*;

#[derive(Clone)]
pub struct Modal {
    element: Element,
    content: Element,
    message_container: Element,
}

impl Modal {
    pub fn new() -> Result<Self, JsValue> {
        let window = window().unwrap();
        let document = window.document().unwrap();
        
        let modal = document.create_element("div")?;
        modal.set_class_name("modal");
        
        let content = document.create_element("div")?;
        content.set_class_name("modal-content");
        
        let message_container = document.create_element("div")?;
        message_container.set_class_name("modal-message");
        
        let close_btn = document.create_element("button")?;
        close_btn.set_class_name("modal-close");
        close_btn.set_inner_html("×");
        
        // Amélioration du gestionnaire d'événements pour le bouton de fermeture
        let modal_ref = modal.clone();
        let close_callback = Closure::wrap(Box::new(move |e: Event| {
            e.prevent_default();
            modal_ref.set_class_name("modal");
        }) as Box<dyn FnMut(Event)>);
        
        // Ajout d'un événement de fermeture en cliquant sur l'arrière-plan
        let modal_ref = modal.clone();
        let outside_click = Closure::wrap(Box::new(move |e: Event| {
            let target = e.target().unwrap();
            let element: Element = target.dyn_into().unwrap();
            if element.class_name() == "modal show" {
                modal_ref.set_class_name("modal");
            }
        }) as Box<dyn FnMut(Event)>);
        
        close_btn.add_event_listener_with_callback("click", close_callback.as_ref().unchecked_ref())?;
        modal.add_event_listener_with_callback("click", outside_click.as_ref().unchecked_ref())?;
        
        close_callback.forget();
        outside_click.forget();
        
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

    pub fn show(&self, message: &str) -> Result<(), JsValue> {
        self.message_container.set_inner_html(message);
        self.element.set_attribute("style", "display: flex")?;
        // Petit délai pour permettre à la transition de s'effectuer
        let window = window().unwrap();
        window.request_animation_frame(&js_sys::Function::new_no_args(""))?;
        self.element.set_class_name("modal show");
        Ok(())
    }

    pub fn hide(&self) -> Result<(), JsValue> {
        self.element.set_class_name("modal");
        Ok(())
    }
}
