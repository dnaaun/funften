use crate::components::select::Button;
use crate::include_html;
use leptos::html::*;
use leptos::*;

#[allow(non_snake_case)]
pub fn SelectButton<V: IntoView + std::clone::Clone>(
    cx: Scope,
    btn_text: MaybeSignal<V>,
) -> HtmlElement<Button> {
    let btn_child = MaybeSignal::derive(cx, move || {
        div(cx)
            .classes("flex content-between justify-between items-center")
            .child(div(cx).classes("mr-2").child(btn_text.clone()))
            .child(include_html!(cx, "../../icons/chevron-down.svg").classes("w-4 h-4"))
    });

    Button(cx, btn_child)
}
