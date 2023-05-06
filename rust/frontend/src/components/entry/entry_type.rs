use leptos::*;

use crate::components::{page::EntryTypeState, select::Select};

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
