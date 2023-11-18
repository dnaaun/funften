pub fn use_doc(cx: leptos::Scope) -> yrs::Doc {
    leptos::use_context::<yrs::Doc>(cx).unwrap()
}

thread_local! {
static CUR_DOC: yrs::Doc = yrs::Doc::new();
}
