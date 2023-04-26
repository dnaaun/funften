use frontend::SimpleCounterProps;
use frontend::SimpleCounter;
use leptos::*;

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    mount_to_body(|cx| {
        view! { cx,
            <SimpleCounter
                initial_value=0
                step=1
            />
        }
    })
}
