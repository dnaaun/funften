use std::collections::HashMap;

use html::*;
use leptos::*;

#[allow(non_snake_case)]
pub fn TextInput(
    cx: Scope,
    value: MaybeSignal<String>,
    label_text: Option<MaybeSignal<String>>,
    id: Option<MaybeSignal<String>>,
    input_props: Option<HashMap<String, String>>,
) -> impl IntoView {
    let mut label_el = label_text.map(|txt| {
        label(cx)
            .child(txt)
            .classes("block mb-2 text-sm font-medium text-gray-900 dark:text-white")
    });

    if let Some(id) = id {
        label_el = label_el.map(|el| el.prop("for", move || id.get()));
    }

    let input_el = input(cx)
        .prop("value", move || value.get().to_string())
        .classes(
            "bg-gray-50
border border-gray-300
text-gray-900 text-sm
rounded-lg
focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600
dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
        );

    let input_el = input_props
        .into_iter()
        .flatten()
        .fold(input_el, |input_el, (name, prop)| {
            input_el.prop(name, move || prop.clone())
        });

    div(cx).child(label_el).child(input_el)
}
