use leptos::html::*;
use leptos::*;

use super::calendar::day::length::Length;
use super::calendar::day::period::SubPeriod;
use super::calendar::Calendar;
use super::entry::{Entry, TypeSpecific};

#[allow(non_snake_case)]
pub fn Page(cx: Scope) -> impl IntoView {
    let (days, _) = create_signal(
        cx,
        std::iter::from_fn(|| Some(vec![vec![SubPeriod::Actual(Length(60))]]))
            .cycle()
            .take(7)
            .collect::<Vec<_>>(),
    );
    div(cx)
        .child(Entry(
            cx,
            "Hello".into(),
            TypeSpecific::ActualExecution {
                start: None.into(),
                end: None.into(),
            }
            .into(),
        ))
        .child(Calendar(cx, days()))
}
