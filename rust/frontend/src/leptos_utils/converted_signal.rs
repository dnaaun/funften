use leptos::{*, tracing::info};

/// This uses effects, but not to syncrhonize state with external
/// things like APIs. So I want to avoid it.
pub fn create_converted_signal<T, U>(
    cx: Scope,
    read: ReadSignal<T>,
    write: WriteSignal<T>,
    convert_to: impl Fn(T) -> U + 'static,
    convert_from: impl Fn(U) -> T + 'static,
) -> RwSignal<U>
where
    T: Clone,
    U: Clone,
{
    let converted_signal = create_rw_signal(cx, convert_to(read.get()));

    create_effect(cx, move |_| {
        info!("converting from");
        write(convert_from(converted_signal.get()));
    });

    create_effect(cx, move |_| {
        info!("converting to");
        converted_signal.set(convert_to(read.get()));
    });

    converted_signal
}
