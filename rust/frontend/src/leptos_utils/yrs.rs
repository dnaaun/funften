use leptos::*;

fn create_yrs_cx_signal<T: yrs::types::Observable + Clone>(cx: Scope, mut value: T) -> RwSignal<T> {
    let signal = create_rw_signal(cx, value.clone());

    // Call `update` on the signal to trigger rerendering when the yrs value changes.
    value.observe(move |_, _| signal.update(move |_| ()));

    signal
}
