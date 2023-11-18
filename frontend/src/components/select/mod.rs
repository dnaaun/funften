pub mod select_button;

use crate::components::button::Button;
use std::rc::Rc;

use leptos::*;

use self::select_button::SelectButton;

use super::dropdown::{Dropdown, DropdownItem};

pub struct Select<T>
where
    T: 'static,
{
    pub options: MaybeSignal<Vec<T>>,
    pub selected: Signal<Option<T>>,
    pub on_select: Rc<dyn Fn(T)>,
    pub render_option: Rc<dyn Fn(&T) -> View>,
}

impl<T> Select<T>
where
    T: Clone,
{
    fn view(self, cx: Scope) -> impl IntoView {
        let Select {
            options,
            selected,
            on_select,
            render_option,
        } = self;
        let dropdown_items: Vec<_> = options
            .get()
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let on_select = on_select.clone();

                DropdownItem {
                    text: render_option(&item),
                    key: i.to_string(),
                    on_click: Rc::new(move || on_select(item.clone())),
                }
            })
            .collect();

        Dropdown(
            cx,
            SelectButton {
                text: MaybeSignal::derive(cx, move || {
                    selected
                        .get()
                        .map(|x| render_option(&x).into_view(cx))
                        .unwrap_or("-".into_view(cx))
                }),
            }
            .view(cx),
            dropdown_items.into(),
        )
    }
}

impl<T> IntoView for Select<T>
where
    T: Clone,
{
    fn into_view(self, cx: Scope) -> View {
        self.view(cx).into_view(cx)
    }
}
