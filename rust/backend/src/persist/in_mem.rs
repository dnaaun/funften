use std::{sync::Arc, collections::HashMap};

use parking_lot::{Mutex, MutexGuard};
use yrs::Transact;

use super::{Listener, Persistence};

type Subscription = u32;

pub struct InMemoryPersist {
    // We have to guard it in a mutex because some methods that take &self (namely, transact_mut)
    // require exclusive access.
    doc: Mutex<yrs::Doc>,

    next_listener_num: Subscription,

    update_listeners: Arc<tokio::sync::RwLock<HashMap<Subscription, Listener>>>,
}

impl Persistence for InMemoryPersist {
    type Error = ();
    type Subscription = u32;

    async fn get_doc(&self) -> Result<MutexGuard<'_, yrs::Doc>, Self::Error> {
        Ok(self.doc.lock())
    }

    async fn store_update(&self, update: yrs::Update) -> Result<(), Self::Error> {
        self.doc.lock().transact_mut().apply_update(update);

        let listeners = self.update_listeners.clone();
        tokio::spawn(async move {
            for listener in listeners.read().await.values() {
                listener().await;
            }
        });

        Ok(())
    }

    async fn subscribe_to_updates(&self, listener: Listener) {
        let listener_num = self.next_listener_num;
        let mut update_listeners = self.update_listeners.read().await;
        while update_listeners.contains_key(listener_num)  {
            listener_num = listener_num.overflowing_add(1).0;
            if listener_num == self.next_listener_num {
                panic!("Too many listeners");
            }
        }
        self.update_listeners.write().await.push(listener);
    }


    async fn unsuscribe_to_updates(
        &self,
        subscription: Self::Subscription,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}

impl InMemoryPersist {
    pub fn new() -> Self {
        Self {
            doc: yrs::Doc::new().into(),
            update_listeners: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            next_listener_num: 0,
        }
    }
}
