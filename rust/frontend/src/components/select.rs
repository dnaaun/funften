use crate::{components::button::Button, include_html};
use std::{fmt::Display, rc::Rc};

use html::*;
use leptos::*;

use super::dropdown::{Dropdown, DropdownItem};

#[allow(non_snake_case)]
pub fn Select<T: Display + Clone>(
    cx: Scope,
    options: MaybeSignal<Vec<T>>,
    selected: RwSignal<Option<T>>,
) -> impl IntoView {
    let dropdown_items: Vec<_> = options
        .get()
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, item)| DropdownItem {
            text: item.to_string().into_view(cx),
            key: i.to_string(),
            on_click: Rc::new(move || {
                selected.set(Some(item.clone()));
            }),
        })
        .collect();

    Dropdown(
        cx,
        Button(
            cx,
            MaybeSignal::Dynamic(Signal::derive(cx, move || {
                div(cx)
                    .classes("flex content-between justify-between items-center")
                    .child(move || {
                        div(cx)
                            .classes("mr-2")
                            .child(selected.get().map(|i| i.to_string()).unwrap_or("-".into()))
                    })
                    .child(include_html!(cx, "../icons/caret-down.svg"))
            })),
            || (),
        ),
        dropdown_items.into(),
    )
}
