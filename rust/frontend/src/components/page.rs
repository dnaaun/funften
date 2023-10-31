use wire::state::StatePrelim;
use wire::state::Todo;
use yrs::Map;
use yrs::TextPrelim;
use yrs::Transact;
use yrs_wrappers::yrs_display::YrsDisplay;
use yrs_wrappers::yrs_vec::YrsVecPrelim;
use yrs_wrappers::yrs_wrapper_error::YrsResult;

use chrono::offset::TimeZone;

use chrono::{Duration, Timelike, Utc};
use leptos::html::*;
use leptos::*;
use wire::state::{ActualExecutionPrelim, PlannedExecutionPrelim, TodoPrelim};

use super::calendar::Calendar;
use super::duration::{DurationState, DurationType};
use super::entry::entry_type::EntryTypeState;
use super::topbar::TopBar;
use crate::gui_error::GuiResult;
use crate::leptos_utils::yrs::YrsSignal;
use crate::use_doc::use_doc;

#[derive(Clone, Debug)]
pub struct DraftEntry {
    pub type_: RwSignal<Option<EntryTypeState>>,
    pub text: RwSignal<String>,
    pub start_datetime: RwSignal<String>,
    pub end_datetime: RwSignal<String>,
    pub completed_at: RwSignal<String>,
    pub estimated_duration: DurationState,
    pub parent_todo: RwSignal<Option<Todo>>,
}

impl DraftEntry {
    fn new(cx: Scope) -> Self {
        Self {
            type_: create_rw_signal(cx, Some(EntryTypeState::ActualExecution)),
            text: create_rw_signal(cx, "".into()),
            start_datetime: create_rw_signal(cx, "".into()),
            end_datetime: create_rw_signal(cx, "".into()),
            completed_at: create_rw_signal(cx, "".into()),
            estimated_duration: DurationState {
                duration_amount: create_rw_signal(cx, Default::default()),
                duration_type: create_rw_signal(cx, Some(DurationType::Seconds)),
            },
            parent_todo: create_rw_signal(cx, Default::default()),
        }
    }
}

#[allow(non_snake_case)]
pub fn Page(cx: Scope) -> GuiResult<HtmlElement<Div>> {
    let test_start_date = Utc
        .with_ymd_and_hms(2023, 5, 1, 8, 0, 0)
        .unwrap()
        .naive_utc();

    let entry = DraftEntry::new(cx);
    let start_day = create_rw_signal(cx, test_start_date.date());

    let doc = yrs::Doc::new();
    let root = doc.get_or_insert_map("root");
    leptos::provide_context(cx, doc);

    let state = StatePrelim {
        todos: vec![TodoPrelim {
            title: TextPrelim::new("My only TODO".into()),
            text: TextPrelim::new("My only TODO".into()),
            completed: false.into(),
            created_at: test_start_date.into(),
            estimated_duration: Duration::hours(10).into(),
            planned_executions: vec![PlannedExecutionPrelim {
                start: test_start_date.with_hour(10).unwrap().into(),
                end: test_start_date
                    .with_hour(10)
                    .unwrap()
                    .with_minute(45)
                    .unwrap()
                    .into(),
            }]
            .into(),
            actual_executions: vec![ActualExecutionPrelim {
                start: test_start_date.with_minute(5).unwrap().into(),
                end: Some(
                    test_start_date
                        .with_hour(9)
                        .unwrap()
                        .with_minute(55)
                        .unwrap()
                        .into(),
                ),
            }]
            .into(),
            child_todos: Box::new(YrsVecPrelim::from(vec![TodoPrelim {
                title: TextPrelim::new("My child TODO".into()),
                text: TextPrelim::new("My child TODO".into()),
                completed: false.into(),
                created_at: test_start_date.into(),
                estimated_duration: Duration::hours(10).into(),
                planned_executions: vec![].into(),
                actual_executions: vec![].into(),
                child_todos: Box::new(YrsVecPrelim::from(vec![])).into(),
            }]))
            .into(),
        }]
        .into(),
    };

    let doc = use_doc(cx);
    let mut txn = doc.try_transact_mut().unwrap();
    let state = root.insert(&mut txn, "state", state);
    // let todos = create_rw_signal(cx, state.todos(&txn)?);
    let todos = YrsSignal::new(cx, use_doc(cx), state.todos(&txn)?);
    drop(txn);

    let seven_days = todos.derive(cx, move |todos, txn| {
        tracing::info!("{}", todos.fmt(txn).unwrap());
        Calendar::days_prop_from_todo_datas_and_start_date(&todos, txn, start_day.get())
    });

    // Auto-fill the start and end datetime fields with the start date corresponding to the day
    // that starts the visible calendar.
    create_effect(cx, move |_| {
        let start_date_formatted = start_day.get().format("%Y-%m-%dT01:00").to_string();
        entry.start_datetime.set(start_date_formatted.clone());
        entry
            .end_datetime
            .set(start_date_formatted.replace("01:00", "01:45"));
    });

    let flattened_todos = todos.derive(cx, |todos, txn| {
        let mut flattened_todos = vec![];
        for todo in todos.iter(txn) {
            let todo = todo?;
            let child_todos = todo
                .child_todos(txn)?
                .iter(txn)
                .collect::<YrsResult<Vec<_>>>()?;
            flattened_todos.push(todo);
            flattened_todos.extend(child_todos);
        }
        YrsResult::Ok(flattened_todos)
    });

    Ok(div(cx)
        .child(TopBar {
            entry,
            start_day,
            flattened_todos,
        })
        .child(Calendar {
            seven_days,
            start_day: start_day.into(),
        }))
}
