use std::rc::Rc;

use html::*;
use leptos::{leptos_dom::Each, *};

use super::popover::Popover;

pub trait CloneFnMut: FnMut() + Clone {}

#[derive(Clone)]
pub struct DropdownItem {
    pub text: View,
    pub key: String,
    pub on_click: Rc<dyn Fn()>,
}

/// Two generics instead of the more natural one because `Popover` requires it.
#[allow(non_snake_case)]
pub fn Dropdown<T, H>(
    cx: Scope,
    head: HtmlElement<T>,
    items: MaybeSignal<Vec<DropdownItem>>,
) -> impl IntoView
where
    T: std::ops::Deref<Target = H> + 'static + leptos::html::ElementDescriptor + std::clone::Clone,
    H: std::ops::Deref<Target = web_sys::HtmlElement>,
{
    let items_el = div(cx)
        .classes(
        "z-10 bg-white divide-y divide-gray-100 rounded-lg shadow w-44 dark:bg-gray-700",
    ).child(
            ul(cx)
            .classes("py-2 text-sm text-gray-700 dark:text-gray-200")
            .child(
                Each::new(
                    items,
                    |item| item.key.clone(),
                    |cx, item| {
                        li(cx)
                            .child(
                                a(cx)
                                    .prop("href", "#")
                                    .classes("block px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-600 dark:hover:text-white")
                                    .child(item.text)
                                    .on(ev::click, move |_| (item.on_click)())
                            )
                                
                    }
                    )
            )
        ) ;

    Popover(cx, head, items_el)
}

