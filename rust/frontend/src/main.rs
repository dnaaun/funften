use frontend::components::page::Page;
use leptos::*;
use wasm_bindgen::prelude::JsCast;

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    let root_el = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("root");
    // turn the above from an `Element` to a `HtmlElement`
    let root_el = root_el.unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
    mount_to(root_el, Page);
}
