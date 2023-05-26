use itertools::Itertools;
use leptos::html::*;
use leptos::*;

pub struct Button {
    pub disabled: MaybeSignal<bool>,
}

impl Button {
    pub fn view(self, cx: Scope) -> HtmlElement<leptos::html::Button> {
        let Button { disabled } = self;

        let color_style_class = Signal::derive(cx, move || {
            if !disabled.get() {
                vec![
                    "bg-blue-700",
                    "hover:bg-blue-800",
                    "focus:ring-4",
                    "focus:ring-blue-300",
                    "dark:bg-blue-600",
                    "dark:hover:bg-blue-700",
                    "focus:outline-none",
                    "dark:focus:ring-blue-800",
                ]
            } else {
                vec!["bg-blue-400", "dark:bg-blue-500", "cursor-not-allowed"]
            }
        });

        let style_class = Signal::derive(cx, move || {
            color_style_class
                .get()
                .into_iter()
                .chain([
                    "text-white",
                    "font-medium",
                    "rounded-lg",
                    "text-sm",
                    "px-5",
                    "py-2.5",
                ])
                .collect::<Vec<_>>()
        });

        button(cx)
            .attr("type", "button")
            .prop("disabled", disabled)
            // Using dyn_classes has a bug that omits some classes when changing the classes.
            .attr("class", move || style_class.get().into_iter().join(" "))
    }
}

impl IntoView for Button {
    fn into_view(self, cx: Scope) -> View {
        self.view(cx).into_view(cx)
    }
}
