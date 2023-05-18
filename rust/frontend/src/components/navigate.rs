use chrono::{Duration, NaiveDate};
use leptos::html::*;
use leptos::*;

pub struct Navigate {
    pub start_day: RwSignal<NaiveDate>,
}

impl Navigate {
    pub fn view(self, cx: Scope) -> impl IntoView {
        let start_day = self.start_day;
        div(cx)
            .classes("flex items-stretch justify-between gap-2")
            .child(
                button(cx)
                    .classes("border border-gray-200 rounded-md px-2 py-1")
                    .on(ev::click, move |_| {
                        start_day.set(start_day.get() - Duration::days(7));
                    })
                    .child("Previous Week"),
            )
            .child(
                button(cx)
                    .classes("border border-gray-200 rounded-md px-2 py-1")
                    .on(ev::click, move |_| {
                        start_day.set(start_day.get() + Duration::days(7));
                    })
                    .child("Next Week"),
            )
    }
}

impl IntoView for Navigate {
    fn into_view(self, cx: Scope) -> View {
        self.view(cx).into_view(cx)
    }
}
