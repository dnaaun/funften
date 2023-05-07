use chrono::offset::TimeZone;
use leptos::tracing::info;
use std::iter::from_fn;

use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};
use leptos::html::*;
use leptos::*;
use uuid::Uuid;
use wire::state::{ActualExecutionData, PlannedExecutionData, TodoData};

use crate::components::calendar::day::DayProps;

use super::calendar::day::length::FiveMins;
use super::calendar::day::period::{Period, PeriodState};
use super::calendar::day::PeriodWithOffset;
use super::calendar::{Calendar, CalendarProps};
use super::duration::{Duration, DurationState, DurationType};
use super::entry::entry_type::EntryTypeState;
use super::topbar::TopBar;

pub struct DraftEntryState {
    pub type_: RwSignal<Option<EntryTypeState>>,
    pub text: RwSignal<String>,
    pub start_datetime: RwSignal<String>,
    pub end_datetime: RwSignal<String>,
    pub completed_at: RwSignal<String>,
    pub estimated_duration: DurationState,
}

impl DraftEntryState {
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

#[allow(non_snake_case)]
pub fn Page(cx: Scope) -> HtmlElement<Div> {
    let test_start_date = Utc.with_ymd_and_hms(2023, 5, 1, 8, 0, 0).unwrap();

    let todos = vec![TodoData {
        id: Uuid::new_v4(),
        text: "My only TODO".into(),
        completed: false,
        created_at: test_start_date,
        estimated_duration: Duration::hours(10),
        planned_executions: vec![PlannedExecutionData {
            id: Uuid::new_v4(),
            start: test_start_date,
            end: test_start_date.with_minute(30).unwrap(),
        }],
        actual_executions: vec![ActualExecutionData {
            id: Uuid::new_v4(),
            start: test_start_date.with_minute(15).unwrap(),
            end: None,
        }],
        child_todos: Box::new(vec![]),
    }];

    let start_day = create_rw_signal(cx, test_start_date.naive_utc().date());

    let (calendar_props, _) = create_signal(
        cx,
        CalendarProps::init_from_todo_datas_and_start_date(todos, start_day.read_only()),
    );

    create_effect(cx, move |_| {
        info!("calendar_props: {:?}", calendar_props.get().days);
    });


    let draft_entry = DraftEntryState::new(cx);

    div(cx)
        .child(TopBar(cx, draft_entry))
        .child(Calendar(cx, calendar_props.get()))
}
