use leptos::*;
use leptos_dom::html::div;

use super::length::Length;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SubPeriod {
    Actual(Length),
    Planned(Length),
}

#[allow(non_snake_case)]
pub fn Period(cx: Scope, periods: Vec<SubPeriod>) -> impl IntoView {
    div(cx)
        .prop("style", "minwidth: 80px")
        .classes(
            "bg-blue-500 text-white
p-1
rounded-md
shadow-sm
shadow-gray-400
",
        )
        .child("s=kjjkup")
}
