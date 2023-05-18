use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;
use wire::state::StatePrelim;
use yrs::Map;
use yrs::TextPrelim;
use yrs::Transact;
use yrs::TransactionMut;
use yrs_wrappers::yrs_vec::YrsVecPrelim;

use chrono::offset::TimeZone;

use chrono::{Duration, Timelike, Utc};
use leptos::html::*;
use leptos::*;
use wire::state::{ActualExecutionPrelim, PlannedExecutionPrelim, TodoPrelim};

use crate::gui_error::GuiResult;

use super::calendar::Calendar;
use super::duration::{DurationState, DurationType};
use super::entry::entry_type::EntryTypeState;
use super::topbar::TopBar;

#[derive(Clone)]
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

pub type TransactionMutContext = Rc<RefCell<TransactionMut<'static>>>;

#[allow(non_snake_case)]
pub fn Page(cx: Scope) -> GuiResult<HtmlElement<Div>> {
    let test_start_date = Utc
        .with_ymd_and_hms(2023, 5, 1, 8, 0, 0)
        .unwrap()
        .naive_utc();

    let entry = DraftEntry::new(cx);
    let start_day = create_rw_signal(cx, test_start_date.date());

    let root = DOC.get_or_insert_map("root");
    // Rc<RefCell<>> to make TransactMut Clone, so that I can leptos::provide_context it.
    let rc_refcell_txn: TransactionMutContext = Rc::new(RefCell::new(DOC.deref().transact_mut()));
    leptos::provide_context(cx, rc_refcell_txn.clone());

    let txn = leptos::use_context::<TransactionMutContext>(cx).unwrap();
    let state = StatePrelim {
        todos: vec![TodoPrelim {
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
            child_todos: Box::new(YrsVecPrelim::from(vec![])).into(),
        }]
        .into(),
    };

    let state = root.insert(&mut txn.borrow_mut(), "state", state);
    let seven_days = Signal::derive(cx, move || {
        let todos = state.todos(txn.borrow().deref());
        Calendar::days_prop_from_todo_datas_and_start_date(
            &todos?,
            txn.borrow_mut().deref_mut(),
            start_day.get(),
        )
    });

    // Auto-fill the start and end datetime fields with the start date corresponding to the day
    // that starts the visible calendar.
    create_effect(cx, move |_| {
        let start_date_formatted = start_day.get().format("%Y-%m-%dT08:00").to_string();
        entry.start_datetime.set(start_date_formatted.clone());
        entry
            .end_datetime
            .set(start_date_formatted.replace("08:00", "08:45"));
    });

    Ok(div(cx).child(TopBar { entry, start_day }).child(Calendar {
        seven_days,
        start_day: start_day.into(),
    }))
}
