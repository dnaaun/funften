use leptos::html::div;
use frontend::components::calendar::{
    day::{length::Length, period::SubPeriod},
    Calendar,
};
use leptos::*;
use wire::state::State;

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    mount_to_body(|cx| {
        let (days, _) = create_signal(
            cx,
            std::iter::from_fn(|| Some(vec![vec![SubPeriod::Actual(Length(60))]]))
                .cycle()
                .take(7)
                .collect::<Vec<_>>(),
        );

        div(cx)
            .child(div(cx).classes("h-9"))
            .child(Calendar(cx, days())).into_view(cx)
    })
}
