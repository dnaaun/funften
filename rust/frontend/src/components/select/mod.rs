pub mod select_button;

use crate::components::button::Button;
use std::{fmt::Display, rc::Rc};

use html::*;
use leptos::*;

use self::select_button::SelectButton;

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
        SelectButton(
            cx,
            MaybeSignal::derive(cx, move || {
                selected.get().map(|i| i.to_string()).unwrap_or("-".into())
            }),
        ),
        dropdown_items.into(),
    )
}
