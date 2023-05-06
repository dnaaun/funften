pub mod entry_type;

use leptos::html::*;
use leptos::*;

use self::entry_type::EntryType;

use super::page::DraftEntryState;
use super::text_input::TextInput;

#[allow(non_snake_case)]
pub fn Entry(cx: Scope, draft_entry: DraftEntryState) -> impl IntoView {
    div(cx)
        .classes("flex items-stretch w-full mt-2")
        .child(EntryType(cx, draft_entry.type_))
        .child(TextInput(
            cx,
            draft_entry.text.into(),
            None,
            None,
            Some(
                [("style".to_string(), "min-width: 30rem".to_string())]
                    .into_iter()
                    .collect(),
            ),
        ))
}
