use leptos::html::div;
use leptos::leptos_dom::Each;
use leptos::*;
use std::ops::Deref;

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
    pub period_with_offsets: Signal<Vec<PeriodWithOffset>>,
}

#[allow(non_snake_case)]
pub fn Day(cx: Scope, props: DayProps) -> impl IntoView {
    div(cx)
        .prop("style", "height: 96em")
        .classes(
            "items-stretch
flex-grow
            border-l border-gray-200
            pl-1 pr-2
            relative
            ",
        )
        .child(Each::new(
            props.period_with_offsets,
            |p| p.clone(),
            |cx, p| {
                div(cx)
                    .prop(
                        "style",
                        format!("position: absolute; top: {}rem", p.offset.deref()),
                    )
                    .classes("w-11/12")
                    .child(Period(cx, PeriodProps { period: p.period }))
            },
        ))
}
