use leptos::html::*;
use leptos::*;

use crate::include_html;

use super::entry::Entry;
use super::page::DraftEntryState;
use super::popover::Popover;

#[allow(non_snake_case)]
pub fn TopBar(cx: Scope, draft_entry: DraftEntryState) -> HtmlElement<Div> {
    div(cx)
        .classes("flex justify-between items-center w-full h-14 px-4 border-b border-gray-200")
        .child(Popover(
            cx,
            include_html!(cx, "../../icons/add.svg").classes(
                "w-10 h-10 cursor-pointer rounded-full hover:bg-gray-100 p-1 shadow-md text-gray-500 hover:text-gray-700",
            ),
            Entry(cx, draft_entry),
        ))
}
