use std::iter::repeat;

use crate::components::calendar::day::Day;
use chrono::{NaiveDate, Timelike};
use leptos::*;
use leptos::{leptos_dom::Each, tracing::info};
use leptos_dom::html::div;
use wire::state::TodoData;

use self::day::{length::FiveMins, period::PeriodState, DayProps, PeriodWithOffset};
pub mod day;

#[derive(Clone, Debug)]
pub struct CalendarProps {
    pub start_day: ReadSignal<NaiveDate>,
    pub days: Vec<DayProps>,
}

impl CalendarProps {
    pub fn init_from_todo_datas_and_start_date(
        todo_datas: impl AsRef<Vec<TodoData>>,
        start_day_signal: ReadSignal<NaiveDate>,
    ) -> Self {
        let mut days = Vec::from_iter(
            repeat(DayProps {
                period_with_offsets: Vec::new(),
            })
            .take(7),
        );

        let start_day = start_day_signal.get();
        let end_day = start_day + chrono::Duration::days(7);

        for todo_data in todo_datas.as_ref() {
            todo_data
                .planned_executions
                .iter()
                .filter(|ae| {
                    let ae_start = ae.start.naive_utc().date();
                    info!("Every variable: ae_start: {:?}, end_day: {:?}, start_day >= ae_start && end_day <= ae_start: {:?} ", ae_start, end_day, start_day >= ae_start && end_day <= ae_start);
                    start_day <= ae_start && end_day >= ae_start
                })
                .for_each(|ae| {
                    let day_index = (ae.start.naive_utc().date() - start_day).num_days() as usize;

                    let six_am_on_start_day =
                        (|| ae.start.with_hour(6)?.with_minute(0)?.with_second(0))().unwrap();

                    info!("SUP: {:?}", ae.start.naive_utc().date());

                    days[day_index].period_with_offsets.push({
                        PeriodWithOffset {
                            period: PeriodState::Planned(FiveMins::from(ae.end - ae.start)),
                            offset: FiveMins::from(ae.start - six_am_on_start_day),
                        }
                    })
                });

            todo_data
                .actual_executions
                .iter()
                .filter(|ae| {
                    let naive_start = ae.start.naive_utc().date();
                    start_day >= naive_start && end_day <= naive_start
                })
                .for_each(|ae| {
                    let day_index = (ae.start.naive_utc().date() - start_day).num_days() as usize;

                    let six_am_on_start_day =
                        (|| ae.start.with_hour(6)?.with_minute(0)?.with_second(0))().unwrap();

                    let period = match ae.end {
                        Some(end) => PeriodState::Actual(FiveMins::from(end - ae.start)),
                        None => PeriodState::ActualUnbonded,
                    };
                    days[day_index].period_with_offsets.push({
                        PeriodWithOffset {
                            period,
                            offset: FiveMins::from(ae.start - six_am_on_start_day),
                        }
                    })
                });

            let days_from_child_todos = CalendarProps::init_from_todo_datas_and_start_date(
                &todo_data.child_todos,
                start_day_signal,
            )
            .days;

            days.iter_mut()
                .zip(days_from_child_todos)
                .for_each(|(day, day_from_child_todo)| {
                    day.period_with_offsets
                        .extend(day_from_child_todo.period_with_offsets)
                })
        }

        Self {
            start_day: start_day_signal,
            days,
        }
    }
}

#[allow(non_snake_case)]
pub fn Calendar(cx: Scope, props: CalendarProps) -> impl IntoView {
    div(cx)
        .classes(
            "flex items-stretch
w-full",
        )
        .child(Each::new(
            move || props.days.clone(),
            |day| day.clone(),
            Day,
        ))
}
