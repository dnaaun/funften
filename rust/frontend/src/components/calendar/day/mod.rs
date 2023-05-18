use chrono::{Duration, NaiveDate};
use leptos::html::div;
use leptos::leptos_dom::Each;
use leptos::*;
use once_cell::sync::Lazy;
use std::ops::Deref;
use yrs_wrappers::yrs_wrapper_error::YrsResult;

use crate::gui_error::GuiResult;

use self::length::TimeLength;
use self::period::{Period, PeriodProps, PeriodState};

pub mod length;
pub mod period;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PeriodWithOffset {
    pub period: PeriodState,
    pub offset: TimeLength,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DayProps {
    pub day: Signal<NaiveDate>,
    pub period_with_offsets: Signal<YrsResult<Vec<PeriodWithOffset>>>,
}

#[allow(non_snake_case)]
pub fn Day(
    cx: Scope,
    DayProps {
        day,
        period_with_offsets,
    }: DayProps,
) -> GuiResult<impl IntoView> {
    static TWENTY_FOUR_HOURS_LENGTH: Lazy<usize> =
        Lazy::new(|| *TimeLength::from(Duration::hours(24)).deref());

    let period_with_offsets = period_with_offsets.get()?;
    Ok(div(cx)
        .classes("items-stretch flex-grow relative")
        .child(
            div(cx)
                .classes("h-20 flex flex-col items-center gap-y-2 mt-2")
                .child(div(cx).child(move || day.get().format("%a").to_string()))
                .child(div(cx).child(move || day.get().format("%e").to_string())),
        )
        .child(
            div(cx)
                .prop(
                    "style",
                    format!("height: {}rem", TWENTY_FOUR_HOURS_LENGTH.deref()),
                )
                .classes("items-stretch flex-grow relative border-l border-gray-200")
                .child(Each::new(
                    move || period_with_offsets.clone(),
                    |p| p.clone(),
                    |cx, p| {
                        div(cx)
                            .prop(
                                "style",
                                format!(
                                    "width: 95%; position: absolute; top: {}rem",
                                    p.offset.deref()
                                ),
                            )
                            .child(Period(cx, PeriodProps { period: p.period }))
                    },
                )),
        ))
}
