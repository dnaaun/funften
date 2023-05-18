use std::iter::repeat;
use std::ops::Deref;

use crate::{components::calendar::day::Day, gui_error::GuiResult};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use leptos::*;
use leptos_dom::html::div;
use wire::state::Todo;
use yrs_wrappers::{yrs_vec::YrsVec, yrs_wrapper_error::YrsResult};

use self::day::{length::TimeLength, period::PeriodState, DayProps, PeriodWithOffset};

pub mod day;

#[derive(Clone, Debug)]
pub struct Calendar {
    pub start_day: Signal<NaiveDate>,
    pub days: Signal<YrsResult<Vec<Vec<PeriodWithOffset>>>>,
}

impl Calendar {
    pub fn days_prop_from_todo_datas_and_start_date(
        todos: &YrsVec<Todo>,
        txn: &impl yrs::ReadTxn,
        start_day: NaiveDate,
    ) -> YrsResult<Vec<Vec<PeriodWithOffset>>> {
        let mut days: Vec<Vec<PeriodWithOffset>> = repeat(Vec::new()).take(7).collect();

        let end_day = start_day + chrono::Duration::days(7);
        let within_week = |d| d >= start_day && d < end_day;
        let midnight_before = |d: NaiveDateTime| -> NaiveDateTime {
            d.date()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap()
                .naive_utc()
        };

        for todo in todos.iter(txn) {
            todo.planned_executions(txn)?
                .iter(txn)
                .map(|pe| {
                    let start = pe.start(txn)?.date();
                    Ok((pe, start))
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .filter_map(|(pe, start)| if within_week(start) { Some(pe) } else { None })
                .map(|ae| {
                    let day_index = (ae.start(txn)?.date() - start_day).num_days() as usize;
                    days[day_index].push({
                        PeriodWithOffset {
                            period: PeriodState::Planned(TimeLength::from(
                                *ae.end(txn)? - *ae.start(txn)?,
                            )),
                            offset: TimeLength::from(
                                *ae.start(txn)? - midnight_before(*ae.start(txn)?),
                            ),
                        }
                    });
                    Ok(())
                })
                .collect::<Result<(), _>>()?;

            todo.actual_executions(txn)?
                .iter(txn)
                .map(|ae| {
                    let start = ae.start(txn)?.date();
                    Ok((ae, start))
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .filter_map(|(pe, start)| if within_week(start) { Some(pe) } else { None })
                .map(|ae| {
                    let start = ae.start(txn)?;
                    let day_index = (start.date() - start_day).num_days() as usize;

                    let period = match ae.end(txn) {
                        Some(end) => PeriodState::Actual(TimeLength::from(*end? - *start)),
                        None => PeriodState::ActualUnbonded,
                    };
                    days[day_index].push({
                        PeriodWithOffset {
                            period,
                            offset: TimeLength::from(*start - midnight_before(*start)),
                        }
                    });

                    Ok(())
                })
                .collect::<Result<(), _>>()?;

            let days_from_child_todos = Self::days_prop_from_todo_datas_and_start_date(
                todo.child_todos(txn)?.deref().deref(),
                txn,
                start_day,
            );

            days.iter_mut()
                .zip(days_from_child_todos?)
                .for_each(|(day, day_from_child_todo)| day.extend(day_from_child_todo))
        }

        Ok(days)
    }

    pub fn view(self, cx: Scope) -> GuiResult<impl IntoView> {
        Ok(div(cx).classes("flex items-stretch w-full").child(move || {
            GuiResult::<_>::Ok(
                (0..self.days.get()?.len())
                    .map(|i| {
                        Day(
                            cx,
                            DayProps {
                                period_with_offsets: Signal::derive(cx, move || {
                                    self.days.get().map(|d| d[i].clone())
                                }),
                                day: Signal::derive(cx, move || {
                                    self.start_day.get() + chrono::Duration::days(i as i64)
                                }),
                            },
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        }))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Timelike, Utc};
    use wire::state::{ActualExecutionPrelim, PlannedExecutionPrelim, StatePrelim, TodoPrelim};
    use yrs::{Map, TextPrelim, Transact};
    use yrs_wrappers::{ybox::YBox, yrs_wrapper_error::YrsResult};

    use crate::components::calendar::day::{
        length::TimeLength, period::PeriodState, PeriodWithOffset,
    };

    use super::Calendar;

    #[test]
    fn test_calendar_init_from_todo_datas() -> YrsResult<()> {
        let start_date = Utc.with_ymd_and_hms(2023, 5, 1, 8, 0, 0).unwrap();

        let state_prelim = StatePrelim {
            todos: vec![TodoPrelim {
                text: TextPrelim::new("My only TODO".into()),
                completed: false.into(),
                created_at: start_date.naive_utc().into(),
                estimated_duration: Duration::hours(10).into(),
                planned_executions: vec![PlannedExecutionPrelim {
                    start: start_date.naive_utc().into(),
                    end: (|| start_date.with_hour(9)?.with_minute(45))()
                        .unwrap()
                        .naive_utc()
                        .into(),
                }]
                .into(),
                actual_executions: vec![ActualExecutionPrelim {
                    start: start_date.with_minute(5).unwrap().naive_utc().into(),
                    end: None,
                }]
                .into(),
                child_todos: YBox::new(
                    vec![TodoPrelim {
                        text: TextPrelim::new("My child TODO".into()),
                        completed: false.into(),
                        created_at: (start_date + Duration::days(1)).naive_utc().into(),
                        estimated_duration: Duration::hours(7).into(),
                        planned_executions: vec![PlannedExecutionPrelim {
                            start: (start_date + Duration::days(1)).naive_utc().into(),
                            end: ((|| start_date.with_hour(9)?.with_minute(45))().unwrap()
                                + Duration::days(1))
                            .naive_utc()
                            .into(),
                        }]
                        .into(),
                        actual_executions: vec![].into(),
                        child_todos: YBox::new(vec![].into()),
                    }]
                    .into(),
                ),
            }]
            .into(),
        };

        let doc = yrs::Doc::new();
        let map = doc.get_or_insert_map("map");
        let mut txn = doc.transact_mut();
        let state = map.insert(&mut txn, "state", state_prelim);

        let days = Calendar::days_prop_from_todo_datas_and_start_date(
            &state.todos(&txn)?,
            &mut txn,
            start_date.naive_utc().date(),
        )?;

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
        );

        Ok(())
    }
}
