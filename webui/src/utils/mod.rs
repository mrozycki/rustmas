mod dummy;

#[allow(unused)]
pub use dummy::Dummy;

use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlFormElement, HtmlInputElement, HtmlSelectElement};

pub fn get_form(target: Option<EventTarget>) -> Option<HtmlFormElement> {
    target
        .clone()
        .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
        .and_then(|e| e.form())
        .or(target
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            .and_then(|e| e.form()))
}
