use std::ops::Deref;
use wasm_bindgen::prelude::{Closure, JsCast};

use leptos::{html::ElementDescriptor, *};

/// I don't know how to make this work without the "deref chain" (unless I capitulate and make it
/// non-generic at all).
#[allow(non_snake_case)]
pub fn Popover<Head, HeadWebSys, Body, BodyWebSys>(
    cx: Scope,
    head: HtmlElement<Head>,
    body: HtmlElement<Body>,
) -> impl IntoView
where
    Head: Deref<Target = HeadWebSys> + 'static + ElementDescriptor + std::clone::Clone,
    HeadWebSys: Deref<Target = web_sys::HtmlElement>,
    Body: Deref<Target = BodyWebSys> + 'static + ElementDescriptor + std::clone::Clone,
    BodyWebSys: Deref<Target = web_sys::HtmlElement>,
{
    let head_ref = create_node_ref::<Head>(cx);
    let body_ref = create_node_ref::<Body>(cx);

    let (body_style, set_body_style) =
        create_signal(cx, "display: none; position: absolute;".to_string());
    let body = body.attr("style", body_style).node_ref(body_ref);

    let head_on_click = move |_| {
        let el = head_ref
            .get()
            .expect("ref should be set if click handler is called");

        let web_sys_el: &web_sys::HtmlElement = Deref::deref(&el);

        let rect = web_sys_el.get_bounding_client_rect();

        let body_x = rect.x();
        let body_y = rect.y() + rect.height() + 5.0; // 5.0 for now.

        set_body_style(format!(
            "display: block; position: absolute; left: {}px; top: {}px;",
            body_x, body_y
        ));

        // set a click handler on `Window` to close the popover when the user clicks outside of
        // `body`.
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
            let event_target = event.target().unwrap();
            let event_target: &web_sys::Node = event_target.dyn_ref().unwrap();

            let body_el = body_ref
                .get()
                .expect("ref should be set if click handler is called");
            let body_el: &web_sys::Node = Deref::deref(&body_el);

            if !body_el.contains(Some(event_target)) {
                set_body_style("display: none; position: absolute;".to_string());
            }
        });
        document().set_onclick(Some(closure.as_ref().unchecked_ref()));

        // https://github.com/rustwasm/wasm-bindgen/blob/0753bec4c6f51d7e27b82c357e65cefab3c61dd3/examples/closures/src/lib.rs#L72-L82
        closure.forget();
    };

    let head = head.node_ref(head_ref).on(ev::click, head_on_click);

    Fragment::new(vec![head.into_view(cx), body.into_view(cx)])
}
