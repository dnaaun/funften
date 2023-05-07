use chrono::NaiveDate;
use leptos::html::*;
use leptos::*;

pub struct NavigateProps {
    pub start_day: RwSignal<NaiveDate>,
}

#[allow(non_snake_case)]
pub fn Navigate(cx: Scope, props: NavigateProps) -> impl IntoView {
    div(cx).classes("flex items-stretch justify-between")
}
