use leptos::*;

/// A simple counter component.
///
/// You can use doc comments like this to document your component.
#[component]
pub fn SimpleCounter(
    cx: Scope,
    /// The starting value for the counter
    initial_value: i32,
    /// The change that should be applied each time the button is clicked.
    step: i32,
) -> impl IntoView {
    let (value, set_value) = create_signal(cx, initial_value);

    view! { cx,
            <div class="flex gap-1 items-center">
            <button class="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 mr-2 mb-2 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800" on:click=move |_| set_value(0)>"Clear"</button>
            <button on:click=move |_| set_value.update(|value| *value -= step)>"-1"</button>
            <span>"Value: " {value} "!"</span>
            <button on:click=move |_| set_value.update(|value| *value += step)>"+1"</button>
        </div>
    }
}
