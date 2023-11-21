use std::fmt::Display;
use std::future::IntoFuture;

use futures::Future;
use indexed_db_futures::prelude::*;
use indexed_db_futures::web_sys::wasm_bindgen::JsValue;
use indexed_db_futures::{
    request::OpenDbRequest, web_sys::DomException, IdbDatabase, IdbVersionChangeEvent,
};
use js_sys::wasm_bindgen::JsCast;
use js_sys::{ArrayBuffer, Object, Uint8Array};
use yrs_kvstore_async::{KVEntry, KVStore};

struct IdbStore {
    db_name: String,
    object_store_name: String,
    db: IdbDatabase,
}

impl IdbStore {
    async fn new(db_name: String, object_store: String) -> Result<Self, IdbError> {
        let mut db_req: OpenDbRequest = IdbDatabase::open_u32(&db_name, 1)?;

        let object_store2 = object_store.clone();
        db_req.set_on_upgrade_needed(Some(
            move |evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
                // Check if the object store exists; create it if it doesn't
                if let None = evt.db().object_store_names().find(|n| n == &object_store2) {
                    evt.db().create_object_store(&object_store2)?;
                }
                Ok(())
            },
        ));

        let db: IdbDatabase = db_req.await?;

        Ok(Self {
            db_name,
            object_store_name: object_store,
            db,
        })
    }
}

struct IdbEntry {
    key: Vec<u8>,
    value: Vec<u8>,
}

impl KVEntry for IdbEntry {
    fn key(&self) -> &[u8] {
        &self.key
    }

    fn value(&self) -> &[u8] {
        &self.value
    }
}

struct IdbCursor;

/// TODO: An enum in anticipation of having to support more errors.
#[derive(Debug)]
enum IdbError {
    DomException(DomException),
    InvalidValueInIdb(JsValue),
}

impl From<DomException> for IdbError {
    fn from(e: DomException) -> Self {
        Self::DomException(e)
    }
}

impl std::error::Error for IdbError {}

impl Display for IdbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdbError::DomException(e) => write!(f, "{}", e.message()),
            IdbError::InvalidValueInIdb(e) => {
                let err_text = match e.clone().dyn_into::<Object>() {
                    Ok(obj) => obj.to_string().as_string().unwrap(),
                    Err(e) => "InvalidValueInIdb: could not convert error to Object".to_owned(),
                };
                write!(f, "{}", err_text)
            }
        }
    }
}

impl futures::Stream for IdbCursor {
    type Item = IdbEntry;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        todo!()
    }
}

impl<'a> KVStore<'a> for IdbStore {
    type Error = IdbError;

    type Cursor = IdbCursor;

    type Entry = IdbEntry;

    type Return = Vec<u8>;

    async fn get(&self, key: &[u8]) -> Result<Option<Self::Return>, Self::Error> {
        let db = &self.db;
        let txn = db.transaction_on_one(&self.object_store_name)?;
        let store = txn.object_store(&self.object_store_name)?;

        let key = js_sys::Uint8Array::from(key);
        let result = store
            .get(&key)?
            .await?
            .map(|v| v.dyn_into::<Uint8Array>())
            .transpose()
            .map_err(|e| IdbError::InvalidValueInIdb(e))?
            .map(|v| v.to_vec());
        Ok(result)
    }

    async fn upsert(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        let db = &self.db;
        let txn = db
            .transaction_on_one_with_mode(&self.object_store_name, IdbTransactionMode::Readwrite)?;
        let store = txn.object_store(&self.object_store_name)?;
        let key = js_sys::Uint8Array::from(key);
        let value = js_sys::Uint8Array::from(value);
        store.put_key_val(&key, &value)?;

        txn.await.into_result()?;

        Ok(())
    }

    async fn remove(&self, key: &[u8]) -> Result<(), Self::Error> {
        let db = &self.db;
        let txn = db
            .transaction_on_one_with_mode(&self.object_store_name, IdbTransactionMode::Readwrite)?;
        let store = txn.object_store(&self.object_store_name)?;
        let key = js_sys::Uint8Array::from(key);
        store.delete(&key)?;

        txn.await.into_result()?;

        Ok(())
    }

    async fn remove_range(&self, from: &[u8], to: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }

    async fn iter_range(&self, from: &[u8], to: &[u8]) -> Result<Self::Cursor, Self::Error> {
        todo!()
    }

    async fn peek_back(&self, key: &[u8]) -> Result<Option<Self::Entry>, Self::Error> {
        todo!()
    }
}
