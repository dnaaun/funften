use crate::components::select::Button;
use crate::include_html;
use leptos::html::*;
use leptos::*;

pub struct SelectButton<V>
where
    V: IntoView + std::clone::Clone + 'static,
{
    pub text: MaybeSignal<V>,
}

impl<V: IntoView + std::clone::Clone> SelectButton<V> {
    pub fn view(self, cx: Scope) -> HtmlElement<html::Button> {
        let btn_child = div(cx)
            .classes("flex content-between justify-between items-center")
            .child(div(cx).classes("mr-2").child(self.text.clone()))
            .child(include_html!(cx, "../../icons/chevron-down.svg").classes("w-4 h-4"));

        Button { disabled: false.into() }.view(cx).child(btn_child)
    }
}

impl<V> IntoView for SelectButton<V>
where
    V: IntoView + std::clone::Clone + 'static,
{
    fn into_view(self, cx: Scope) -> View {
        self.view(cx).into_view(cx)
    }
}
