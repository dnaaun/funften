pub mod entry_type;

use leptos::html::*;
use leptos::*;

use self::entry_type::{EntryType, EntryTypeState};

use super::duration::Duration;
use super::page::DraftEntryState;
use super::text_input::TextInput;

#[allow(non_snake_case)]
pub fn Entry(cx: Scope, draft_entry: DraftEntryState) -> HtmlElement<Div> {
    let DraftEntryState {
        type_,
        text,
        start_datetime,
        end_datetime,
        completed_at,
        estimated_duration,
    } = draft_entry;

    let datetime_input_props = Some(
        [("type".to_string(), "datetime-local".to_string())]
            .into_iter()
            .collect(),
    );

    let entry_type_specific = move || {
        type_.get().map(|t| match t {
            EntryTypeState::PlannedExecution | EntryTypeState::ActualExecution => vec![
                TextInput(
                    cx,
                    start_datetime.into(),
                    Some("From".into()),
                    datetime_input_props.clone(),
                ),
                TextInput(
                    cx,
                    end_datetime.into(),
                    Some("To".into()),
                    datetime_input_props.clone(),
                ),
            ]
            .into_view(cx),
            EntryTypeState::Todo => vec![
                Duration(cx, estimated_duration.clone()).into_view(cx),
                TextInput(
                    cx,
                    completed_at.into(),
                    Some("Completed At".into()),
                    datetime_input_props.clone(),
                )
                .into_view(cx),
            ]
            .into_view(cx),
        })
    };

    div(cx)
        .classes("flex flex-wrap items-center gap-2 py-2 bg-white rounded-md shadow-md p-4")
        .child(EntryType(cx, type_))
        .child(TextInput(
            cx,
            text.into(),
            None,
            Some(
                [("style".to_string(), "min-width: 30rem".to_string())]
                    .into_iter()
                    .collect(),
            ),
        ))
        .child(entry_type_specific)
}
