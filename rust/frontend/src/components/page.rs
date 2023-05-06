use std::fmt::Display;

use leptos::html::*;
use leptos::*;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;

use super::calendar::day::length::Length;
use super::calendar::day::period::SubPeriod;
use super::calendar::Calendar;
use super::entry::Entry;

#[derive(Clone)]
pub enum EntryTypeState {
    PlannedExecution,
    ActualExecution,
    Todo,
}

impl Display for EntryTypeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryTypeState::PlannedExecution => write!(f, "Planned Execution"),
            EntryTypeState::ActualExecution => write!(f, "Actual Execution"),
            EntryTypeState::Todo => write!(f, "Todo"),
        }
    }
}

pub struct DraftEntryState {
    pub type_: RwSignal<Option<EntryTypeState>>,
    pub text: RwSignal<String>,
    pub start: RwSignal<Option<DateTime<Utc>>>,
    pub end: RwSignal<Option<DateTime<Utc>>>,
    pub completed: RwSignal<Option<DateTime<Utc>>>,
    pub estimated_duration: RwSignal<Option<Duration>>,
}

#[allow(non_snake_case)]
pub fn Page(cx: Scope) -> impl IntoView {
    let (days, _) = create_signal(
        cx,
        std::iter::from_fn(|| Some(vec![vec![SubPeriod::Actual(Length(60))]]))
            .cycle()
            .take(7)
            .collect::<Vec<_>>(),
    );

    let draft_entry = DraftEntryState {
        type_: create_rw_signal(cx, Some(EntryTypeState::Todo)),
        text: create_rw_signal(cx, "".to_string()),
        start: create_rw_signal(cx, None),
        end: create_rw_signal(cx, None),
        completed: create_rw_signal(cx, None),
        estimated_duration: create_rw_signal(cx, None),
    };

    div(cx)
        .child(Entry(cx, draft_entry))
        .child(Calendar(cx, days()))
}
