use leptos::html::*;
use leptos::*;

use super::select::Select;
use super::text_input::TextInput;

#[derive(derive_more::Display, Clone)]
pub enum DurationType {
    Minutes,
    Seconds,
}

#[derive(Clone)]
pub struct DurationState {
    pub duration_amount: RwSignal<String>,
    pub duration_type: RwSignal<Option<DurationType>>,
}

#[allow(non_snake_case)]
pub fn Duration(cx: Scope, state: DurationState) -> impl IntoView {
    use DurationType::*;

    div(cx)
        .classes("flex items-center gap-x-2")
        .child(Select(
            cx,
            vec![Minutes, Seconds].into(),
            state.duration_type
        ))
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
