pub mod entry_type;

use std::rc::Rc;

use crate::components::button::Button;
use leptos::html::*;
use leptos::tracing::info;
use leptos::*;

use self::entry_type::{EntryType, EntryTypeState};

use super::duration::Duration;
use super::page::DraftEntry;
use super::text_input::TextInput;

#[derive(Clone)]
pub struct EntryProps {
    pub entry: DraftEntry,
    pub on_save: Rc<dyn Fn(DraftEntry)>,
}

#[allow(non_snake_case)]
pub fn Entry(cx: Scope, draft_entry: DraftEntry) -> HtmlElement<Div> {
    let DraftEntry {
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
        .classes("flex flex-col gap-x-2 gap-y-2 py-2 bg-white rounded-md shadow-md p-4")
        .child(div(cx).classes("shrink").child(EntryType(cx, type_)))
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
        .child(
            div(cx).classes("shrink").child(
                Button(cx)
                    .on(ev::click, move |_| {
                        info!("Add button clicked");
                    })
                    .child("Add"),
            ),
        )
}
