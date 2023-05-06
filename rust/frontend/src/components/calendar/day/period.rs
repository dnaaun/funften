use leptos::*;
use leptos_dom::html::div;

use super::length::Length;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Period {
    Actual(Length),
    Planned(Length),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PeriodProps {
    pub period: Period,
}

#[allow(non_snake_case)]
pub fn Period(cx: Scope, props: PeriodProps) -> impl IntoView {
    div(cx).prop("style", "min-height: 2rem").classes(
        "bg-blue-500 text-white
p-1
rounded-md
shadow-sm
shadow-gray-400
",
    )
}
