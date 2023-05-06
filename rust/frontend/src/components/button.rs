use leptos::html::*;
use leptos::*;

#[allow(non_snake_case)]
pub fn Button<V: IntoView + std::clone::Clone>(
    cx: Scope,
    text: MaybeSignal<V>,
) -> HtmlElement<Button> {
    button(cx)
        .prop(
        "type","button")
        .classes("text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800")
        .child(text)
}
