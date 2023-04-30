use crate::components::calendar::day::period::Period;
use leptos::html::div;
use leptos::leptos_dom::Each;
use leptos::*;

use self::period::SubPeriod;
pub mod length;
pub mod period;

#[allow(non_snake_case)]
pub fn Day(cx: Scope, periods: Vec<Vec<SubPeriod>>) -> impl IntoView {
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
            move || periods.clone(),
            |period| period.clone(),
            Period,
        ))
}
