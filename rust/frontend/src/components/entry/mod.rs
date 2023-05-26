use crate::use_doc::use_doc;
use crate::utils::date::parse_input_datetime;
use chrono::NaiveDateTime;
use yrs::GetString;
use yrs::Transact;
pub mod entry_type;

use std::rc::Rc;
use wire::state::{ActualExecutionPrelim, PlannedExecutionPrelim, Todo};

use crate::components::button::Button;
use leptos::html::*;
use leptos::*;
use yrs_wrappers::yrs_wrapper_error::YrsResult;

use self::entry_type::{EntryType, EntryTypeState};

use super::duration::Duration;
use super::page::DraftEntry;
use super::select;
use super::text_input::TextInput;

#[derive(Clone, Debug)]
pub enum NewEntryToSave {
    PlannedExecution {
        start_datetime: NaiveDateTime,
        end_datetime: NaiveDateTime,
        parent_todo: Todo,
    },
    ActualExecution {
        start_datetime: NaiveDateTime,

        // This should at least be Option<NaiveDateTime>, and ideally should be
        // enum ParsedInput<T> {
        //   Valid(T),
        //   PresentButInvalid(String),
        //   Missing,
        // }
        end_datetime: NaiveDateTime,
        parent_todo: Todo,
    },
}

#[derive(Clone)]
pub struct Entry {
    pub entry: DraftEntry,
    pub on_save: Rc<dyn Fn(DraftEntry)>,
    pub flattened_todos: Signal<YrsResult<Vec<Todo>>>,
}

impl Entry {
    pub fn view(self, cx: Scope) -> HtmlElement<Div> {
        let DraftEntry {
            type_,
            text,
            start_datetime,
            end_datetime,
            completed_at,
            estimated_duration,
            parent_todo,
        } = self.entry;

        let datetime_input_props = Some(
            [("type".to_string(), "datetime-local".to_string())]
                .into_iter()
                .collect(),
        );

        let new_entry_to_save = Signal::derive(cx, move || {
            let data = (
                type_.get(),
                parse_input_datetime(&start_datetime.get()),
                parse_input_datetime(&end_datetime.get()),
                parent_todo.get(),
            );

            match data {
                (Some(type_), Some(start_datetime), Some(end_datetime), Some(parent_todo)) => {
                    match type_ {
                        EntryTypeState::PlannedExecution => {
                            Some(NewEntryToSave::PlannedExecution {
                                start_datetime,
                                end_datetime,
                                parent_todo,
                            })
                        }
                        EntryTypeState::ActualExecution => Some(NewEntryToSave::ActualExecution {
                            start_datetime,
                            end_datetime,
                            parent_todo,
                        }),
                        EntryTypeState::Todo => None,
                    }
                }
                _ => None,
            }
        });

        let entry_type_specific = move || {
            type_.get().map(|t| match t {
                EntryTypeState::PlannedExecution | EntryTypeState::ActualExecution => {
                    let select_el = select::Select {
                        options: self.flattened_todos.get()?.into(),
                        selected: parent_todo.into(),
                        on_select: Rc::new(move |item| parent_todo.set(Some(item.clone()))),
                        render_option: Rc::new(move |i| {
                            let doc = use_doc(cx);
                            let txn = doc.transact();
                            i.text(&txn).map(|t| t.get_string(&txn)).into_view(cx)
                        }),
                    };

                    YrsResult::Ok(
                        vec![
                            select_el.into_view(cx),
                            TextInput(
                                cx,
                                start_datetime.into(),
                                Some("From".into()),
                                datetime_input_props.clone(),
                            )
                            .into_view(cx),
                            TextInput(
                                cx,
                                end_datetime.into(),
                                Some("To".into()),
                                datetime_input_props.clone(),
                            )
                            .into_view(cx),
                        ]
                        .into_view(cx),
                    )
                }
                EntryTypeState::Todo => Ok(vec![
                    TextInput(
                        cx,
                        text.into(),
                        None,
                        Some(
                            [("style".to_string(), "min-width: 30rem".to_string())]
                                .into_iter()
                                .collect(),
                        ),
                    )
                    .into_view(cx),
                    Duration(cx, estimated_duration.clone()).into_view(cx),
                    TextInput(
                        cx,
                        completed_at.into(),
                        Some("Completed At".into()),
                        datetime_input_props.clone(),
                    )
                    .into_view(cx),
                ]
                .into_view(cx)),
            })
        };

        div(cx)
            .classes("flex flex-col gap-x-2 gap-y-2 py-2 bg-white rounded-md shadow-md p-4")
            .child(div(cx).classes("shrink").child(EntryType(cx, type_)))
            .child(entry_type_specific)
            .child(
                div(cx).classes("shrink").child(
                    Button {
                        disabled: Signal::derive(cx, move || new_entry_to_save.get().is_none())
                            .into(),
                    }
                    .view(cx)
                    .child("Add")
                    .on(ev::click, move |_| {
                        let doc = use_doc(cx);
                        let mut txn = doc.try_transact_mut().unwrap();
                        let new_entry_to_save = new_entry_to_save
                            .get()
                            .expect("Button should be disabled otherwise.");
                        cx.batch(move || {
                            match new_entry_to_save {
                                NewEntryToSave::PlannedExecution {
                                    start_datetime,
                                    end_datetime,
                                    parent_todo,
                                } => {
                                    let executions = parent_todo.planned_executions(&txn)?;
                                    executions.push(
                                        &mut txn,
                                        PlannedExecutionPrelim {
                                            start: start_datetime.into(),
                                            end: end_datetime.into(),
                                        },
                                    );
                                }
                                NewEntryToSave::ActualExecution {
                                    start_datetime,
                                    end_datetime,
                                    parent_todo,
                                } => {
                                    let executions = parent_todo.actual_executions(&txn)?;
                                    executions.push(
                                        &mut txn,
                                        ActualExecutionPrelim {
                                            start: start_datetime.into(),
                                            end: Some(end_datetime.into()),
                                        },
                                    );
                                }
                            };
                            YrsResult::Ok(())
                        })
                        .unwrap();
                    }),
                ),
            )
    }
}

impl IntoView for Entry {
    fn into_view(self, cx: Scope) -> View {
        self.view(cx).into_view(cx)
    }
}
