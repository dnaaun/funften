use std::ops::Deref;
use leptos::{*, tracing::info};
use leptos_dom::html::div;

use super::length::TimeLength;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PeriodState {
    ActualUnbonded,
    Actual(TimeLength),
    Planned(TimeLength),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PeriodProps {
    pub period: PeriodState,
}

#[allow(non_snake_case)]
pub fn Period(cx: Scope, props: PeriodProps) -> impl IntoView {
    let len = match props.period {
        PeriodState::ActualUnbonded => None,
        PeriodState::Actual(l) => Some(l),
        PeriodState::Planned(l) => Some(l),
    };

    let style = match len {
        None => "height: 0.5rem".into(),
        Some(l) => format!("height: {}rem", l.deref()),
    };

    info!("style: {:?}", style);

    div(cx).prop("style", style).classes(
        "bg-blue-500 text-white
p-1
rounded-md
shadow-sm
shadow-gray-400
",
    )
}
