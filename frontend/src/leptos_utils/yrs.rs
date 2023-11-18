use std::{cell::RefCell, clone::Clone, rc::Rc};
use yrs::Transact;

use leptos::*;
use yrs::TransactionMut;

#[derive(Clone)]
pub struct YrsSignal<T: yrs::types::DeepObservable + Clone + 'static> {
    doc: yrs::Doc,

    inner: RwSignal<T>,

    /// I think, if I didn't want ot make YrsSignal::derive take &self, I could avoid using
    /// RefCell. But I want to just get it to work first.
    derived_updaters: Rc<RefCell<Vec<Box<dyn Fn(T, &TransactionMut<'_>)>>>>,
}

impl<T: yrs::types::DeepObservable + Clone + 'static> YrsSignal<T> {
    pub fn new(cx: Scope, doc: yrs::Doc, mut value: T) -> YrsSignal<T> {
        let inner = create_rw_signal(cx, value.clone());
        let derived_updaters: Rc<RefCell<Vec<Box<dyn Fn(T, &TransactionMut<'_>)>>>> =
            Rc::new(RefCell::new(vec![]));

        // Call `update` on the signal to trigger rerendering when the yrs value changes.
        let derived_updaters2 = derived_updaters.clone();
        let value2 = value.clone();
        let subscription = value.observe_deep(move |txn, _| {
            tracing::info!("Yrs value changed, triggering rerender");
            inner.update(|_| ());
            for updater in derived_updaters2.borrow().iter() {
                updater(value2.clone(), txn);
            }
        });

        // Maybe there's a way to tie the lifetime of the subscription to that of the signal, but
        // for now we'll have to tie it to the life time of the Scope.
        store_value(cx, subscription);

        YrsSignal {
            doc,
            inner,
            derived_updaters,
        }
    }

    pub fn derive_yrs<N, F>(&self, cx: Scope, f: F) -> YrsSignal<N>
    where
        F: Fn(T, &TransactionMut<'_>) -> N + 'static + Clone,
        N: Clone + std::convert::AsMut<yrs::types::Branch>,
    {
        let value = f(self.inner.get(), &mut self.doc.try_transact_mut().unwrap());

        let inner = create_rw_signal(cx, value);

        let f = Rc::new(f);

        let doc2 = self.doc.clone();
        let original_inner = self.inner.clone();
        let f2 = f.clone();
        create_effect(cx, move |none_if_first| {
            if let Some(_) = none_if_first {
                let value = f2(original_inner.get(), &doc2.try_transact_mut().unwrap());
                inner.set(value);
            }
        });

        let signal = YrsSignal {
            inner,
            derived_updaters: Rc::new(RefCell::new(vec![])),
            doc: self.doc.clone(),
        };

        self.derived_updaters
            .borrow_mut()
            .push(Box::new(move |new, txn| signal.inner.set(f(new, txn))));

        signal
    }

    pub fn derive<N, F>(&self, cx: Scope, f: F) -> Signal<N>
    where
        F: Fn(T, &TransactionMut<'_>) -> N + 'static + Clone,
    {
        let value = f(self.inner.get(), &mut self.doc.try_transact_mut().unwrap());

        let signal = create_rw_signal(cx, value);

        let f = Rc::new(f);

        let doc2 = self.doc.clone();
        let original_inner = self.inner.clone();
        let f2 = f.clone();
        create_effect(cx, move |none_if_first| {
            if let Some(_) = none_if_first {
                let value = f2(original_inner.get(), &doc2.try_transact_mut().unwrap());
                signal.set(value);
            }
        });

        self.derived_updaters
            .borrow_mut()
            .push(Box::new(move |new, txn| signal.set(f(new, txn))));

        signal.into()
    }

    pub fn get(&self) -> T {
        self.inner.get()
    }
}
