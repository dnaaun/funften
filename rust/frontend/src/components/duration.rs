use leptos::html::*;
use leptos::*;

use super::select::Select;
use super::text_input::TextInput;
use chrono::Duration;

#[derive(derive_more::Display, Clone)]
pub enum DurationType {
    Minutes,
    Seconds,
}

#[allow(non_snake_case)]
pub fn Duration(cx: Scope, value: RwSignal<Duration>) -> impl IntoView {
    use DurationType::*;

    let duration_type = create_rw_signal(
        cx,
        Some(if value.get().num_minutes() > 0 {
            Minutes
        } else {
            Seconds
        }),
    );

    let duration_text = create_rw_signal(cx, "".to_owned());

    create_effect(cx, move |_| {
        let duration_type = duration_type.get();
        let duration_text = duration_text.get();

        let duration = match duration_type {
            Some(DurationType::Minutes) => Duration::minutes(duration_text.parse().unwrap_or(0)),
            Some(DurationType::Seconds) => Duration::seconds(duration_text.parse().unwrap_or(0)),
            None => Duration::seconds(0),
        };

        value.set(duration);
    });

    div(cx)
        .classes("flex items-center gap-x-2")
        .child(Select(cx, vec![Minutes, Seconds].into(), duration_type))
        .child(TextInput(
            cx,
            duration_text.into(),
            None,
            Some(
                [("type".to_owned(), "number".to_owned())]
                    .into_iter()
                    .collect(),
            ),
        ))
}
