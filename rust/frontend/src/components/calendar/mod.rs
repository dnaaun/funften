use std::iter::repeat;

use crate::components::calendar::day::Day;
use chrono::{DateTime, NaiveDate, Utc};
use leptos::*;
use leptos_dom::html::div;
use wire::state::{State, Todo};
use yrs::TransactionMut;

use self::day::{length::TimeLength, period::PeriodState, DayProps, PeriodWithOffset};

pub mod day;

#[derive(Clone, Debug)]
pub struct CalendarProps {
    pub start_day: Signal<NaiveDate>,
    pub days: Signal<Vec<Vec<PeriodWithOffset>>>,
}

impl CalendarProps {
    pub fn days_prop_from_todo_datas_and_start_date(
        todos: Vec<Todo>,
        txn: &TransactionMut,
        start_day: NaiveDate,
    ) -> Vec<Vec<PeriodWithOffset>> {
        let mut days: Vec<Vec<PeriodWithOffset>> = repeat(Vec::new()).take(7).collect();

        let end_day = start_day + chrono::Duration::days(7);
        let within_week = |d| d >= start_day && d < end_day;
        let midnight_before = |d: DateTime<Utc>| -> DateTime<Utc> {
            d.date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap()
        };

        for todo in todos.iter() {
            todo
                .planned_executions(txn)
                .iter()
                .filter(|e| within_week(e.start(txn).naive_utc().date()))
                .for_each(|ae| {
                    let day_index =
                        (ae.start(txn).naive_utc().date() - start_day).num_days() as usize;
                    days[day_index].push({
                        PeriodWithOffset {
                            period: PeriodState::Planned(TimeLength::from(
                                ae.end(txn) - ae.start(txn),
                            )),
                            offset: TimeLength::from(
                                ae.start(txn) - midnight_before(ae.start(txn)),
                            ),
                        }
                    })
                });

            todo
                .actual_executions(txn)
                .iter()
                .filter(|e| within_week(e.start(txn).naive_utc().date()))
                .for_each(|ae| {
                    let start = ae.start(txn);
                    let day_index = (start.naive_utc().date() - start_day).num_days() as usize;

                    let period = match ae.end(txn) {
                        Some(end) => PeriodState::Actual(TimeLength::from(end - start)),
                        None => PeriodState::ActualUnbonded,
                    };
                    days[day_index].push({
                        PeriodWithOffset {
                            period,
                            offset: TimeLength::from(start - midnight_before(start)),
                        }
                    })
                });

            let days_from_child_todos =
                Self::days_prop_from_todo_datas_and_start_date(todo.child_todos(txn), txn, start_day);

            days.iter_mut()
                .zip(days_from_child_todos)
                .for_each(|(day, day_from_child_todo)| day.extend(day_from_child_todo))
        }

        days
    }
}

#[allow(non_snake_case)]
pub fn Calendar(cx: Scope, props: CalendarProps) -> impl IntoView {
    div(cx).classes("flex items-stretch w-full").child(move || {
        (0..props.days.get().len())
            .map(|i| {
                Day(
                    cx,
                    DayProps {
                        period_with_offsets: Signal::derive(cx, move || {
                            props.days.get()[i].clone()
                        }),
                        day: Signal::derive(cx, move || {
                            props.start_day.get() + chrono::Duration::days(i as i64)
                        }),
                    },
                )
            })
            .collect::<Vec<_>>()
    })
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use chrono::Timelike;
    use chrono::{Duration, Utc};
    use uuid::Uuid;
    use wire::state::State;
    use wire::state::{ActualExecutionData, PlannedExecutionData, TodoData};
    use yrs::Transact;

    use crate::components::calendar::day::length::TimeLength;
    use crate::components::calendar::day::period::PeriodState;
    use crate::components::calendar::day::PeriodWithOffset;

    use super::CalendarProps;

    #[test]
    fn test_calendar_init_from_todo_datas() {
        let start_date = Utc.with_ymd_and_hms(2023, 5, 1, 8, 0, 0).unwrap();

        let todos = vec![TodoData {
            id: Uuid::new_v4(),
            text: "My only TODO".into(),
            completed: false,
            created_at: start_date,
            estimated_duration: Duration::hours(10),
            planned_executions: vec![PlannedExecutionData {
                id: Uuid::new_v4(),
                start: start_date,
                end: (|| start_date.with_hour(9)?.with_minute(45))().unwrap(),
            }],
            actual_executions: vec![ActualExecutionData {
                id: Uuid::new_v4(),
                start: start_date.with_minute(5).unwrap(),
                end: None,
            }],
            child_todos: Box::new(vec![TodoData {
                id: Uuid::new_v4(),
                text: "My child todo".into(),
                completed: false,
                created_at: start_date + Duration::days(1),
                estimated_duration: Duration::hours(7),
                planned_executions: vec![PlannedExecutionData {
                    id: Uuid::new_v4(),
                    start: start_date + Duration::days(1),
                    end: (|| start_date.with_hour(9)?.with_minute(45))().unwrap()
                        + Duration::days(1),
                }],
                actual_executions: vec![],
                child_todos: Box::new(vec![]),
            }]),
        }];

        let doc = yrs::Doc::new();
        let map = doc.get_or_insert_map("map");
        let mut txn = doc.transact_mut();
        let state = State::new(map, &mut txn, todos);

        let days = CalendarProps::days_prop_from_todo_datas_and_start_date(
            state.todos(&mut txn)(txn),
            &mut txn,
            start_date.naive_utc().date(),
        );

        assert!(days[2..].iter().all(|day| day.is_empty()));

        assert_eq!(
            days[0],
            vec![
                PeriodWithOffset {
                    period: PeriodState::Planned(super::day::length::TimeLength::from(
                        Duration::hours(1) + Duration::minutes(45)
                    )),
                    offset: TimeLength::from(Duration::hours(8))
                },
                PeriodWithOffset {
                    period: PeriodState::ActualUnbonded,
                    offset: TimeLength::from(Duration::hours(8) + Duration::minutes(5))
                }
            ]
        );

        assert_eq!(
            days[1],
            vec![PeriodWithOffset {
                period: PeriodState::Planned(super::day::length::TimeLength::from(
                    Duration::hours(1) + Duration::minutes(45)
                )),
                offset: TimeLength::from(Duration::hours(8))
            },]
        )
    }
}
