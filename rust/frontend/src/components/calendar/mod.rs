use crate::components::calendar::day::Day;
use leptos::leptos_dom::Each;
use leptos::*;
use leptos_dom::html::div;

use self::day::DayProps;
pub mod day;

#[derive(Clone)]
pub struct CalendarProps {
    pub days: Vec<DayProps>,
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
