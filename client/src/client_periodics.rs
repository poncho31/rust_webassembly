use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::window;
use std::future::Future;

pub async fn run_async_request<F, Fut>(f: F, interval_seconds: i32)
where
    F: Fn() -> Fut + 'static,
    Fut: Future<Output = ()> + 'static,
{
    // Premier appel immédiat
    f().await;

    // Si interval_seconds > 0, configurer l'appel périodique
    if interval_seconds > 0 {
        let window = window().unwrap();
        let callback = Closure::wrap(Box::new(move || {
            wasm_bindgen_futures::spawn_local(f());
        }) as Box<dyn FnMut()>);

        window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                interval_seconds * 1000,
            )
            .unwrap();
        
        callback.forget();
    }
}

// /// Anime un élément avec une classe CSS à intervalle régulier
// pub async fn animate_element<F>(
//     element_id: &str,
//     animation_class: &str,
//     interval_seconds: i32,
//     condition: F
// ) where F: Fn() -> bool + 'static {
//     let do_animate = || async {
//         if let Some(window) = window() {
//             if let Some(document) = window.document() {
//                 if let Some(element) = document.get_element_by_id(element_id) {
//                     if condition() {
//                         element.class_list().add_1(animation_class).unwrap();
//                         // Retirer la classe après l'animation
//                         element.class_list().remove_1(animation_class).unwrap();
//                     }
//                 }
//             }
//         }
//     };

//     run_async_request(do_animate, interval_seconds).await;
// }

// /// Sauvegarde périodiquement les valeurs d'un formulaire dans le localStorage
// pub async fn auto_save_form(form_id: &str, interval_seconds: i32) {
//     let form_id = form_id.to_string();
    
//     let do_save = move || async {
//         if let Some(window) = window() {
//             if let Some(document) = window.document() {
//                 if let Some(form) = document.get_element_by_id(&form_id) {
//                     let storage = window.local_storage().unwrap().unwrap();
//                     let form_data = web_sys::FormData::new_with_form(&form.dyn_into().unwrap()).unwrap();
//                     storage.set_item(&format!("{}_autosave", form_id), &form_data.to_string()).unwrap();
//                 }
//             }
//         }
//     };

//     run_async_request(do_save, interval_seconds).await;
// }


// /// Vérifie périodiquement si un élément est visible dans le viewport
// pub async fn check_element_visibility(element_id: &str, interval_seconds: i32, mut callback: impl FnMut(bool) + 'static) {
//     let element_id = element_id.to_string();
    
//     let do_check = move || async {
//         if let Some(window) = window() {
//             if let Some(document) = window.document() {
//                 if let Some(element) = document.get_element_by_id(&element_id) {
//                     let rect = element.get_bounding_client_rect();
//                     let is_visible = rect.top() >= 0.0 && rect.bottom() <= window.inner_height().unwrap() as f64;
//                     callback(is_visible);
//                 }
//             }
//         }
//     };

//     run_async_request(do_check, interval_seconds).await;
// }


// /// Surveille périodiquement l'activité de l'utilisateur
// pub async fn monitor_user_activity(timeout_seconds: i32, mut on_inactive: impl FnMut() + 'static) {
//     let last_activity = std::rc::Rc::new(std::cell::RefCell::new(js_sys::Date::now()));
//     let last_activity_clone = last_activity.clone();

//     // Mettre à jour le timestamp à chaque activité
//     let window = window().unwrap();
//     let update_activity = Closure::wrap(Box::new(move || {
//         *last_activity_clone.borrow_mut() = js_sys::Date::now();
//     }) as Box<dyn FnMut()>);

//     window.add_event_listener_with_callback("mousemove", update_activity.as_ref().unchecked_ref()).unwrap();
//     window.add_event_listener_with_callback("keypress", update_activity.as_ref().unchecked_ref()).unwrap();
//     update_activity.forget();

//     // Vérifier l'inactivité
//     let check_activity = move || async move {
//         let now = js_sys::Date::now();
//         if now - *last_activity.borrow() > (timeout_seconds as f64 * 1000.0) {
//             on_inactive();
//         }
//     };

//     run_async_request(check_activity, 1).await;
// }


// /// Adapte le thème en fonction de l'heure de la journée
// pub async fn dynamic_theme_manager(check_interval_seconds: i32) {
//     let do_theme_check = || async {
//         let hour = js_sys::Date::new_0().get_hours();
//         let document = window().unwrap().document().unwrap();
//         let root = document.document_element().unwrap();
        
//         match hour {
//             6..=17 => root.set_attribute("data-theme", "light").unwrap(),
//             _ => root.set_attribute("data-theme", "dark").unwrap(),
//         }
//     };

//     run_async_request(do_theme_check, check_interval_seconds).await;
// }