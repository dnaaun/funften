pub mod entry_type;

use leptos::html::*;
use leptos::*;

use self::entry_type::EntryType;

use super::duration::Duration;
use super::page::{DraftEntryState, EntryTypeState};
use super::text_input::TextInput;

#[allow(non_snake_case)]
pub fn Entry(cx: Scope, draft_entry: DraftEntryState) -> impl IntoView {
    let entry_type_specific = draft_entry.type_.get().map(|t| match t {
        EntryTypeState::PlannedExecution | EntryTypeState::ActualExecution => {
            Fragment::new(vec![]).into_view(cx)
        }
        EntryTypeState::Todo => div(cx)
            .child(Duration(cx, draft_entry.estimated_duration))
            .into_view(cx),
    });

    div(cx)
        .classes("flex items-center gap-2 w-full py-2")
        .child(EntryType(cx, draft_entry.type_))
        .child(TextInput(
            cx,
            draft_entry.text.into(),
            None,
            Some(
                [("style".to_string(), "min-width: 30rem".to_string())]
                    .into_iter()
                    .collect(),
            ),
        ))
        .child(entry_type_specific)
}
