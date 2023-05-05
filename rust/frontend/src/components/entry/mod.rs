pub mod entry_type;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use leptos::html::*;
use leptos::*;

use self::entry_type::EntryType;

use super::text_input::TextInput;

#[derive(Clone)]
pub enum TypeSpecific {
    PlannedExecution {
        start: MaybeSignal<Option<DateTime<Utc>>>,
        end: MaybeSignal<Option<DateTime<Utc>>>,
    },
    ActualExecution {
        start: MaybeSignal<Option<DateTime<Utc>>>,
        end: MaybeSignal<Option<DateTime<Utc>>>,
    },
    Todo {
        completed: MaybeSignal<Option<DateTime<Utc>>>,
        estimated_duration: MaybeSignal<Option<Duration>>,
    },
}

#[allow(non_snake_case)]
pub fn Entry(
    cx: Scope,
    text: MaybeSignal<String>,
    type_specific: ReadSignal<TypeSpecific>,
    set_type_specific: WriteSignal<TypeSpecific>,
) -> impl IntoView {
    div(cx)
        .classes("flex items-stretch w-full mt-2")
        .child(EntryType(cx, type_specific, set_type_specific))
        .child(TextInput(
            cx,
            text,
            None,
            None,
            Some(
                [("style".to_string(), "min-width: 60rem".to_string())]
                    .into_iter()
                    .collect(),
            ),
        ))
}
