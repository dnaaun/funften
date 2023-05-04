use std::rc::Rc;

use crate::components::dropdown::DropdownItem;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use leptos::html::*;
use leptos::*;

use super::dropdown::Dropdown;
use super::select::select_button::SelectButton;

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
    let dropdown_items: Vec<_> = vec![
        DropdownItem {
            text: "Planned Execution".into_view(cx),
            key: "planned_execution".into(),
            on_click: Rc::new(move || match type_specific.get() {
                TypeSpecific::PlannedExecution { .. } => (),
                TypeSpecific::ActualExecution { start, end } => {
                    set_type_specific(TypeSpecific::PlannedExecution { start, end })
                }
                TypeSpecific::Todo { .. } => set_type_specific(TypeSpecific::PlannedExecution {
                    start: None.into(),
                    end: None.into(),
                }),
            }),
        },
        DropdownItem {
            text: "Actual Execution".into_view(cx),
            key: "actual_execution".into(),
            on_click: Rc::new(move || match type_specific.get() {
                TypeSpecific::ActualExecution { .. } => (),
                TypeSpecific::PlannedExecution { start, end } => {
                    set_type_specific(TypeSpecific::ActualExecution { start, end })
                }
                TypeSpecific::Todo { .. } => set_type_specific(TypeSpecific::ActualExecution {
                    start: None.into(),
                    end: None.into(),
                }),
            }),
        },
        DropdownItem {
            text: "Todo".into_view(cx),
            key: "todo".into(),
            on_click: Rc::new(move || match type_specific.get() {
                TypeSpecific::ActualExecution { .. } | TypeSpecific::PlannedExecution { .. } => {
                    set_type_specific(TypeSpecific::Todo {
                        completed: None.into(),
                        estimated_duration: None.into(),
                    })
                }

                TypeSpecific::Todo { .. } => set_type_specific(TypeSpecific::ActualExecution {
                    start: None.into(),
                    end: None.into(),
                }),
            }),
        },
    ];

    let dropdown = Dropdown(
        cx,
        SelectButton(
            cx,
            MaybeSignal::derive(cx, move || {
                match type_specific() {
                    TypeSpecific::PlannedExecution { .. } => "Planned Execution",
                    TypeSpecific::ActualExecution { .. } => "Actual Execution",
                    TypeSpecific::Todo { .. } => "Todo",
                }
                .into_view(cx)
            }),
        ),
        dropdown_items.into(),
    );

    div(cx)
        .classes("flex items-stretch w-full")
        .child(dropdown)
        .child(text)
}
