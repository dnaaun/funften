use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use leptos::html::*;
use leptos::*;

use crate::components::select::Select;

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
    type_specific: RwSignal<TypeSpecific>,
) -> impl IntoView {
    #[derive(Clone)]
    enum EntryType {
        PlannedExecution,
        ActualExecution,
        Todo,
    }

    impl From<&TypeSpecific> for EntryType {
        fn from(type_specific: &TypeSpecific) -> Self {
            match type_specific {
                TypeSpecific::PlannedExecution { .. } => EntryType::PlannedExecution,
                TypeSpecific::ActualExecution { .. } => EntryType::ActualExecution,
                TypeSpecific::Todo { .. } => EntryType::Todo,
            }
        }
    }

    impl std::fmt::Display for EntryType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                EntryType::PlannedExecution => "Planned Execution",
                EntryType::ActualExecution => "Actual Execution",
                EntryType::Todo => "Todo",
            })
        }
    }

    let entry_type = create_rw_signal(cx, Some((&type_specific.get()).into()));

    div(cx)
        .classes("flex items-stretch w-full")
        .child(Select(
            cx,
            vec![
                EntryType::PlannedExecution,
                EntryType::ActualExecution,
                EntryType::Todo,
            ]
            .into(),
            entry_type,
        ))
        .child(text)
}
