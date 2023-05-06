use leptos::*;

use crate::components::select::Select;

#[derive(Clone, derive_more::Display)]
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
    Select(
        cx,
        vec![
            EntryTypeState::PlannedExecution,
            EntryTypeState::ActualExecution,
            EntryTypeState::Todo,
        ]
        .into(),
        type_,
    )
}
