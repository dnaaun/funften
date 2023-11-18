/// Not ideal. Wraps it in a div.
#[macro_export]
macro_rules! include_html {
    ($cx:ident, $string:expr) => {
        leptos::html::div($cx).inner_html(include_str!($string))
    };
}
