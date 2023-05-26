use std::rc::Rc;

use leptos::html::*;
use leptos::*;

use super::select::Select;
use super::text_input::TextInput;

#[derive(derive_more::Display, Clone, Debug)]
pub enum DurationType {
    Minutes,
    Seconds,
}

#[derive(Clone, Debug)]
pub struct DurationState {
    pub duration_amount: RwSignal<String>,
    pub duration_type: RwSignal<Option<DurationType>>,
}

#[allow(non_snake_case)]
pub fn Duration(cx: Scope, state: DurationState) -> impl IntoView {
    use DurationType::*;

    div(cx)
        .classes("flex items-center gap-x-2")
        .child(Select {
            options: vec![Minutes, Seconds].into(),
            selected: state.duration_type.into(),
            on_select: Rc::new(move |i| state.duration_type.set(Some(i))),
            render_option: Rc::new(move |i| i.to_string().into_view(cx)),
        })
        .child(TextInput(
            cx,
            state.duration_amount,
            None,
            Some(
                [("type".to_owned(), "number".to_owned())]
                    .into_iter()
                    .collect(),
            ),
        ))
}
