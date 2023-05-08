use chrono::{Duration, NaiveDate};
use leptos::html::*;
use leptos::*;

pub struct NavigateProps {
    pub start_day: RwSignal<NaiveDate>,
}

#[allow(non_snake_case)]
pub fn Navigate(cx: Scope, NavigateProps { start_day }: NavigateProps) -> impl IntoView {
    div(cx)
        .classes("flex items-stretch justify-between gap-2")
        .child(
            button(cx)
                .classes("border border-gray-200 rounded-md px-2 py-1")
                .on(ev::click, move |_| {
                    start_day.set(start_day.get() - Duration::days(7));
                })
                .child("Previous Week"),
        )
        .child(
            button(cx)
                .classes("border border-gray-200 rounded-md px-2 py-1")
                .on(ev::click, move |_| {
                    start_day.set(start_day.get() + Duration::days(7));
                })
                .child("Next Week"),
        )
}
