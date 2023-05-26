use std::rc::Rc;

use leptos::*;

use crate::components::select::Select;

#[derive(Clone, derive_more::Display, Debug)]
pub enum EntryTypeState {
    #[display(fmt = "Planned Execution")]
    PlannedExecution,

    #[display(fmt = "Actual Execution")]
    ActualExecution,

    #[display(fmt = "Todo")]
    Todo,
}

#[allow(non_snake_case)]
pub fn EntryType(cx: Scope, type_: RwSignal<Option<EntryTypeState>>) -> impl IntoView {
    Select {
        options: vec![
            EntryTypeState::PlannedExecution,
            EntryTypeState::ActualExecution,
            EntryTypeState::Todo,
        ]
        .into(),
        render_option: Rc::new(move |i| i.to_string().into_view(cx)),
        selected: type_.into(),
        on_select: Rc::new(move |item| type_.set(Some(item))),
    }
}
