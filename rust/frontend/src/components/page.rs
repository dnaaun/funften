use std::fmt::Display;

use leptos::html::*;
use leptos::*;

use chrono::Duration;

use super::calendar::day::length::Length;
use super::calendar::day::period::SubPeriod;
use super::calendar::Calendar;
use super::duration::{DurationState, DurationType};
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
    pub start_datetime: RwSignal<String>,
    pub end_datetime: RwSignal<String>,
    pub completed_at: RwSignal<String>,
    pub estimated_duration: DurationState,
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
        type_: create_rw_signal(cx, Some(EntryTypeState::ActualExecution)),
        text: create_rw_signal(cx, "".into()),
        start_datetime: create_rw_signal(cx, "".into()),
        end_datetime: create_rw_signal(cx, "".into()),
        completed_at: create_rw_signal(cx, "".into()),
        estimated_duration: DurationState {
            duration_amount: create_rw_signal(cx, Default::default()),
            duration_type: create_rw_signal(cx, Some(DurationType::Seconds)),
        },
    };

    div(cx)
        .child(Entry(cx, draft_entry))
        .child(Calendar(cx, days()))
}
