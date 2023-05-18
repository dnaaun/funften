use leptos::html::*;
use leptos::*;

use crate::include_html;

use super::entry::Entry;
use super::navigate::{Navigate, NavigateProps};
use super::page::DraftEntry;
use super::popover::Popover;

pub struct TopBar {
    pub draft_entry: DraftEntry,
    pub start_day: RwSignal<chrono::NaiveDate>,
}

impl TopBar {
    pub fn view(self, cx: Scope) -> impl IntoView {
        let TopBar {
            draft_entry,
            start_day,
        } = self;
        div(cx)
        .classes("flex justify-between items-center w-full h-14 px-4 border-b border-gray-200
bg-white z-10")
        .child(Popover(
            cx,
            include_html!(cx, "../../icons/add.svg").classes(
                "w-10 h-10 cursor-pointer rounded-full hover:bg-gray-100 p-1 shadow-md text-gray-500 hover:text-gray-700",
            ),
            Entry(cx, draft_entry),
        ))
            .child(
            Navigate(cx, NavigateProps {
                start_day,
            })
        )
    }
}
