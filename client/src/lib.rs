use core::User;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{window, Document, HtmlElement};
use serde_wasm_bindgen::from_value;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let win = window().unwrap();
    let doc: Document = win.document().unwrap();
    let container = doc.create_element("div")?;
    container.set_id("users");
    doc.body().unwrap().append_child(&container)?;

    let update = Closure::wrap(Box::new(move || {
        let win = window().unwrap();
        let doc = win.document().unwrap();
        let promise = win.fetch_with_str("/api/users");
        spawn_local(async move {
            let resp = JsFuture::from(promise).await.unwrap();
            let resp: web_sys::Response = resp.dyn_into().unwrap();
            let json = JsFuture::from(resp.json().unwrap()).await.unwrap();
            let users: Vec<User> = from_value(json).unwrap();

            if let Some(old) = doc.get_element_by_id("users-list") {
                old.remove();
            }
            let ul = doc.create_element("ul").unwrap();
            ul.set_id("users-list");
            for u in users {
                let li = doc.create_element("li").unwrap()
                    .dyn_into::<HtmlElement>().unwrap();
                li.set_text_content(Some(&format!("{} <{}>", u.name, u.email)));
                ul.append_child(&li).unwrap();
            }
            doc.get_element_by_id("users").unwrap()
               .append_child(&ul).unwrap();
        });
    }) as Box<dyn Fn()>);

    update.as_ref().unchecked_ref::<js_sys::Function>().call0(&JsValue::NULL).ok();
    win.set_interval_with_callback_and_timeout_and_arguments_0(
        update.as_ref().unchecked_ref(), 5000)?;
    update.forget();

    Ok(())
}
