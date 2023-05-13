use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use yrs::Transact;

use chrono::offset::TimeZone;

use chrono::{Duration, Timelike, Utc};
use leptos::html::*;
use leptos::*;
use uuid::Uuid;
use wire::state::{
    ActualExecutionData, PlannedExecutionData, State, TodoData, TransactionMutContext,
};

use super::calendar::{Calendar, CalendarProps};
use super::duration::{DurationState, DurationType};
use super::entry::entry_type::EntryTypeState;
use super::topbar::{TopBar, TopBarProps};

pub struct DraftEntry {
    pub type_: RwSignal<Option<EntryTypeState>>,
    pub text: RwSignal<String>,
    pub start_datetime: RwSignal<String>,
    pub end_datetime: RwSignal<String>,
    pub completed_at: RwSignal<String>,
    pub estimated_duration: DurationState,
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
        }
    }
}

static DOC: Lazy<yrs::Doc> = Lazy::new(|| yrs::Doc::new());

#[allow(non_snake_case)]
pub fn Page(cx: Scope) -> HtmlElement<Div> {
    let test_start_date = Utc.with_ymd_and_hms(2023, 5, 1, 8, 0, 0).unwrap();

    let draft_entry = DraftEntry::new(cx);
    let start_day = create_rw_signal(cx, test_start_date.naive_utc().date());

    let map = DOC.get_or_insert_map("state");
    // Rc<RefCell<>> to make TransactMut Clone, so that I can leptos::provide_context it.
    let rc_refcell_txn: TransactionMutContext = Rc::new(RefCell::new(DOC.deref().transact_mut()));
    leptos::provide_context(cx, rc_refcell_txn.clone());

    let txn = leptos::use_context::<TransactionMutContext>(cx).unwrap();
    let state = State::new(
        map,
        &mut txn.borrow_mut(),
        vec![TodoData {
            id: Uuid::new_v4(),
            text: "My only TODO".into(),
            completed: false,
            created_at: test_start_date,
            estimated_duration: Duration::hours(10),
            planned_executions: vec![PlannedExecutionData {
                id: Uuid::new_v4(),
                start: test_start_date.with_hour(10).unwrap(),
                end: test_start_date
                    .with_hour(10)
                    .unwrap()
                    .with_minute(45)
                    .unwrap(),
            }],
            actual_executions: vec![ActualExecutionData {
                id: Uuid::new_v4(),
                start: test_start_date.with_minute(5).unwrap(),
                end: Some(
                    test_start_date
                        .with_hour(9)
                        .unwrap()
                        .with_minute(55)
                        .unwrap(),
                ),
            }],
            child_todos: Box::new(vec![]),
        }],
    );

    let cur_seven_days = Signal::derive(cx, move || {
        let todos = state.todos(txn.borrow_mut().deref_mut());
        CalendarProps::days_prop_from_todo_datas_and_start_date(
            todos,
            txn.borrow_mut().deref_mut(),
            start_day.get(),
        )
    });

    // Auto-fill the start and end datetime fields with the start date corresponding to the day
    // that starts the visible calendar.
    create_effect(cx, move |_| {
        let start_date_formatted = start_day.get().format("%Y-%m-%dT08:00").to_string();
        draft_entry.start_datetime.set(start_date_formatted.clone());
        draft_entry
            .end_datetime
            .set(start_date_formatted.replace("08:00", "08:45"));
    });

    div(cx)
        .child(TopBar(
            cx,
            TopBarProps {
                draft_entry,
                start_day,
            },
        ))
        .child(Calendar(
            cx,
            CalendarProps {
                days: cur_seven_days,
                start_day: start_day.into(),
            },
        ))
}
