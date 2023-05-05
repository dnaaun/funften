use leptos::*;

use std::rc::Rc;

use crate::components::{
    dropdown::{Dropdown, DropdownItem},
    select::select_button::SelectButton,
};

use super::TypeSpecific;

#[allow(non_snake_case)]
pub fn EntryType(
    cx: Scope,
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

    let dropdown_head_text = MaybeSignal::derive(cx, move || {
        match type_specific() {
            TypeSpecific::PlannedExecution { .. } => "Planned Execution",
            TypeSpecific::ActualExecution { .. } => "Actual Execution",
            TypeSpecific::Todo { .. } => "Todo",
        }
        .into_view(cx)
    });

    Dropdown(
        cx,
        SelectButton(cx, dropdown_head_text).attr("style", "min-width: 12rem"),
        dropdown_items.into(),
    )
}
