use js_sys::{ArrayBuffer, Promise};
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::IdbObjectStoreParameters;
use web_sys::{IdbDatabase, IdbObjectStore, IdbTransaction, IdbTransactionMode};

use yrs::Doc;

pub const PREFERRED_TRIM_SIZE: u32 = 500;

pub struct IndexeddbPersistence {
    doc: Doc,
    name: String,
    db: Rc<RefCell<Option<IdbDatabase>>>,
    dbref: u32,
    dbsize: u32,
    destroyed: bool,
    synced: bool,
}

impl IndexeddbPersistence {
    pub async fn new(name: &str, doc: Doc) -> Self {
        let mut indexeddb_persistence = IndexeddbPersistence {
            doc,
            name: name.to_string(),
            db: Rc::new(RefCell::new(None)),
            dbref: 0,
            dbsize: 0,
            destroyed: false,
            synced: false,
        };

        indexeddb_persistence.init().await;

        indexeddb_persistence
    }

    async fn init(&mut self) {
        let window = web_sys::window().expect("global window does not exist");
        let indexed_db = window
            .indexed_db()
            .ok()
            .flatten()
            .expect("IndexedDB not available");
        let open_request = indexed_db
            .open_with_u32(self.name.as_str(), 1)
            .expect("Failed to open IndexedDB");

        let db = self.db.clone();

        let onupgradeneeded_callback =
            Closure::wrap(Box::new(move |event: web_sys::IdbVersionChangeEvent| {
                let target = event.target().expect("Failed to get event target");
                let request = target
                    .dyn_into::<web_sys::IdbRequest>()
                    .expect("Failed to cast target to IdbRequest");
                let db = request
                    .result()
                    .expect("Failed to get request result")
                    .dyn_into::<IdbDatabase>()
                    .unwrap();

                // Create object stores
                db.create_object_store_with_optional_parameters(
                    "updates",
                    IdbObjectStoreParameters::new()
                        .key_path(Some(&JsValue::from("id")))
                        .auto_increment(true),
                )
                .expect("Failed to create 'updates' object store");
                db.create_object_store("custom")
                    .expect("Failed to create 'custom' object store");
            }) as Box<dyn FnMut(_)>);

        open_request.set_onupgradeneeded(Some(onupgradeneeded_callback.as_ref().unchecked_ref()));
        onupgradeneeded_callback.forget();

        let db = self.db.clone();
        let onsuccess_callback = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let target = event.target().expect("Failed to get event target");
            let request = target
                .dyn_into::<web_sys::IdbRequest>()
                .expect("Failed to cast target to IdbRequest");
            let db = request
                .result()
                .expect("Failed to get request result")
                .dyn_into::<IdbDatabase>()
                .unwrap();
        }) as Box<dyn FnMut(_)>);

        open_request.set_onsuccess(Some(onsuccess_callback.as_ref().unchecked_ref()));
        onsuccess_callback.forget();

        let onerror_callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            // Handle error
        }) as Box<dyn FnMut(_)>);

        open_request.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        let future = JsFuture::from(Promise::new(&mut |resolve, _reject| {
            resolve.set_on_onsuccess(Some(onsuccess_callback.as_ref().unchecked_ref()));
        }));

        future.await.expect("Failed to initialize IndexedDB");
    }

    async fn fetch_updates(&self) {
        let db = self.db.borrow();
        if let Some(db) = db.as_ref() {
            let transaction = db
                .transaction_with_str_and_mode("updates", IdbTransactionMode::Readonly)
                .unwrap();
            let object_store = transaction.object_store("updates").unwrap();
            let request = object_store.get_all().unwrap();
            let future = JsFuture::from(request);
            let result = future.await.unwrap();

            // Process updates, apply them to the Yrs Doc
        }
    }

    async fn store_update(&self, update: &[u8]) {
        let db = self.borrow();
        let transaction = db
            .transaction_with_str_and_mode("updates", IdbTransactionMode::Readwrite)
            .unwrap();
        let object_store = transaction.object_store("updates").unwrap();
        let array_buffer = ArrayBuffer::new(update.len() as u32);
        js_sys::Uint8Array::new(&array_buffer).copy_from(update);
        let _ = object_store
            .add_with_key(&array_buffer, &JsValue::from(self.dbref))
            .unwrap();

        self.dbref += 1;
        self.dbsize += 1;
        if self.dbsize > PREFERRED_TRIM_SIZE {
            self.trim_db().await;
        }
    }

    async fn trim_db(&self) {
        let db = self.db.borrow();
        if let Some(db) = db.as_ref() {
            let transaction = db
                .transaction_with_str_and_mode("updates", IdbTransactionMode::Readwrite)
                .unwrap();
            let object_store = transaction.object_store("updates").unwrap();

            for key in 0..(self.dbsize - PREFERRED_TRIM_SIZE) {
                let _ = object_store.delete(&JsValue::from(key)).unwrap();
            }

            self.dbsize = PREFERRED_TRIM_SIZE;
        }
    }

    async fn destroy_db(&self) {
        let window = web_sys::window().expect("global window does not exist");
        let indexed_db = window.indexed_db().expect("IndexedDB not available");
        let delete_request = indexed_db
            .delete_database(self.name.as_str())
            .expect("Failed to delete IndexedDB");

        let future = JsFuture::from(delete_request);
        future.await.expect("Failed to delete IndexedDB");
    }
}
