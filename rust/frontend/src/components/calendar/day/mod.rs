use std::ops::Deref;
use leptos::html::div;
use leptos::leptos_dom::Each;
use leptos::*;

use self::length::TimeLength;
use self::period::{Period, PeriodProps, PeriodState};

pub mod length;
pub mod period;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PeriodWithOffset {
    pub period: PeriodState,
    pub offset: TimeLength,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DayProps {
    pub period_with_offsets: Vec<PeriodWithOffset>,
}

#[allow(non_snake_case)]
pub fn Day(cx: Scope, props: DayProps) -> impl IntoView {
    div(cx)
        .prop("style", "height: 96em")
        .classes(
            "flex flex-col
            flex-grow
            border-l border-gray-200
            pl-1 pr-2
            ",
        )
        .child(Each::new(
            move || props.period_with_offsets.clone(),
            |period_with_offsets| period_with_offsets.clone(),
            |cx, item| {
                div(cx)
                    .prop("style", format!("margin-top: {}rem", item.offset.deref()))
                    .child(Period(
                        cx,
                        PeriodProps {
                            period: item.period,
                        },
                    ))
            },
        ))
}
