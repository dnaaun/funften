use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use leptos::html::*;
use leptos::*;
use std::rc::Rc;

use super::button::Button;
use super::dropdown::Dropdown;
use super::dropdown::DropdownItem;

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
    type_specific: MaybeSignal<TypeSpecific>,
) -> impl IntoView {
    div(cx)
        .classes("flex items-stretch w-full")
        .child(Dropdown(
            cx,
            Button(cx, text.clone(), || ()),
            vec![
                DropdownItem {
                    text: "Planned Execution".into_view(cx),
                    key: "planned_execution".into(),
                    on_click: Rc::new(|| {
                        tracing::info!("planned_execution");
                    }),
                },
                DropdownItem {
                    text: "Actual Execution".into_view(cx),
                    key: "actual_execution".into(),
                    on_click: Rc::new(|| {
                        tracing::info!("actual_execution");
                    }),
                },
            ]
            .into(),
        ))
        .child(text)
}
