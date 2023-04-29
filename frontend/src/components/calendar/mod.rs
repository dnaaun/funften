use crate::components::calendar::day::period::SubPeriod;
use crate::components::calendar::day::Day;
use leptos::leptos_dom::Each;
use leptos::*;
use leptos_dom::html::div;
pub mod day;

#[allow(non_snake_case)]
pub fn Calendar(cx: Scope, days: Vec<Vec<Vec<SubPeriod>>>) -> impl IntoView {
    div(cx).classes("flex items-stretch
w-full")
        .child(Each::new(
        move || days.clone(),
        |day| day.clone(),
        Day,
    ))
}
