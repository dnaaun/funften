use leptos::html::*;
use leptos::*;
use std::rc::Rc;
use wire::state::Todo;
use yrs_wrappers::yrs_wrapper_error::YrsResult;

use crate::include_html;

use super::entry::Entry;
use super::navigate::Navigate;
use super::page::DraftEntry;
use super::popover::Popover;

pub struct TopBar {
    pub entry: DraftEntry,
    pub start_day: RwSignal<chrono::NaiveDate>,
    pub flattened_todos: Signal<YrsResult<Vec<Todo>>>,
}

impl TopBar {
    pub fn view(self, cx: Scope) -> impl IntoView {
        let TopBar {
            entry,
            start_day,
            flattened_todos,
        } = self;
        div(cx)
            .classes(
                "flex justify-between items-center w-full h-14 px-4 border-b border-gray-200
bg-white z-10",
            )
            .child(Popover(
                cx,
                include_html!(cx, "../../icons/add.svg").classes(
                    "w-10 h-10 cursor-pointer rounded-full hover:bg-gray-100 p-1 shadow-md text-gray-500 hover:text-gray-700",
                ),
                Entry {
                    entry,
                    flattened_todos,
                    on_save: Rc::new(|_| {}),
                }.view(cx)
                ,
            ))
            .child(Navigate { start_day })
    }
}

impl IntoView for TopBar {
    fn into_view(self, cx: Scope) -> View {
        self.view(cx).into_view(cx)
    }
}
