use std::fmt::Display;
use std::task::Poll;

use futures::FutureExt;
use indexed_db_futures::prelude::*;
use indexed_db_futures::web_sys::wasm_bindgen::JsValue;
use indexed_db_futures::web_sys::IdbKeyRange;
use indexed_db_futures::{
    request::OpenDbRequest, web_sys::DomException, IdbDatabase, IdbVersionChangeEvent,
};
use js_sys::wasm_bindgen::JsCast;
use js_sys::{Object, Uint8Array};
use yrs_kvstore_async::{KVEntry, KVStore};

pub struct IdbStore<'a> {
    object_store: IdbObjectStore<'a>,
}

impl<'a> IdbStore<'a> {
    pub fn new(object_store: IdbObjectStore<'a>) -> Self {
        Self { object_store }
    }

    pub async fn prepare_db(
        db_name: &str,
        object_store_name: &str,
    ) -> Result<IdbDatabase, DomException> {
        let mut db_req: OpenDbRequest = IdbDatabase::open_u32(&db_name, 1)?;

        let object_store2 = object_store_name.to_owned();
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

        Ok(db)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct IdbEntry {
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

pub struct IdbStream<'a> {
    cursor: Option<IdbCursorWithValue<'a, IdbObjectStore<'a>>>,
}

#[derive(Debug)]
pub enum IdbError {
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
                    Err(_) => format!("InvalidValueInIdb: could not convert error to Object",),
                };
                write!(f, "{}", err_text)
            }
        }
    }
}

impl<'a> futures::Stream for IdbStream<'a> {
    type Item = IdbEntry;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let cursor = match self.cursor.as_ref() {
            None => return Poll::Ready(None),
            Some(cursor) => cursor,
        };

        let key = match cursor.key() {
            None => return Poll::Ready(None),
            Some(key) => key,
        };

        let value = cursor.value();

        // For some reason, I get ArrayBuffer for the key instead of a Uint8Array, so I gotta
        // do this instead of dyn_into.
        let key = Uint8Array::new(&key).to_vec();
        let value = value.dyn_into::<Uint8Array>().unwrap().to_vec();

        let continue_cursor_res = cursor.advance(1);
        match continue_cursor_res {
            Ok(mut fut) => {
                cx.waker().wake_by_ref();
                let poll_result = fut.poll_unpin(cx);
                match poll_result {
                    Poll::Ready(result) => {
                        result.unwrap();
                    }
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                }
            }
            Err(err) => {
                // The error below is expected when we reach the end of the cursor.
                // https://developer.mozilla.org/en-US/docs/Web/API/IDBCursor/continue#invalidstateerror
                if err.name() == "InvalidStateError" {
                    return Poll::Ready(None);
                } else {
                    panic!("Unexpected error: {}", err.to_string());
                }
            }
        }

        Poll::Ready(Some(IdbEntry { key, value }))
    }
}

impl<'a> KVStore<'a> for IdbStore<'a> {
    type Error = IdbError;

    type Cursor = IdbStream<'a>;

    type Entry = IdbEntry;

    type Return = Vec<u8>;

    async fn get(&self, key: &[u8]) -> Result<Option<Self::Return>, Self::Error> {
        let key = js_sys::Uint8Array::from(key);
        let result = self
            .object_store
            .get(&key)?
            .await?
            .map(|v| v.dyn_into::<Uint8Array>())
            .transpose()
            .map_err(|e| IdbError::InvalidValueInIdb(e))?
            .map(|v| v.to_vec());
        Ok(result)
    }

    async fn upsert(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        let key = js_sys::Uint8Array::from(key);
        let value = js_sys::Uint8Array::from(value);
        self.object_store.put_key_val(&key, &value)?;

        Ok(())
    }

    async fn remove(&self, key: &[u8]) -> Result<(), Self::Error> {
        let key = js_sys::Uint8Array::from(key);
        self.object_store.delete(&key)?;
        Ok(())
    }

    async fn remove_range(&self, from: &[u8], to: &[u8]) -> Result<(), Self::Error> {
        let from = js_sys::Uint8Array::from(from);
        let to = js_sys::Uint8Array::from(to);
        let range = IdbKeyRange::bound(
            &from.dyn_into::<JsValue>().unwrap(),
            &to.dyn_into::<JsValue>().unwrap(),
        )
        .unwrap();
        let cursor = self.object_store.open_cursor_with_range(&range)?.await?;

        if let Some(cursor) = cursor {
            let items = cursor.into_vec(0).await?;

            for item in items {
                self.object_store.delete(item.key())?;
            }
        }

        Ok(())
    }

    async fn iter_range<'b>(
        &'a self,
        from: &'b [u8],
        to: &'b [u8],
    ) -> Result<IdbStream<'a>, Self::Error> {
        let from = js_sys::Uint8Array::from(from)
            .dyn_into::<JsValue>()
            .unwrap();
        let to = js_sys::Uint8Array::from(to).dyn_into::<JsValue>().unwrap();

        let range = IdbKeyRange::bound(&from, &to).unwrap();

        let cursor = self.object_store.open_cursor_with_range(&range)?.await?;

        Ok(IdbStream { cursor })
    }

    async fn peek_back(&self, key: &[u8]) -> Result<Option<Self::Entry>, Self::Error> {
        let to = js_sys::Uint8Array::from(key);
        let range = IdbKeyRange::upper_bound(&to.dyn_into::<JsValue>().unwrap()).unwrap();
        let cursor = self
            .object_store
            .open_cursor_with_range_and_direction_owned(range, IdbCursorDirection::Prev)?
            .await?;
        let cursor = match cursor {
            None => return Ok(None),
            Some(cursor) => cursor,
        };
        let key = match cursor.key() {
            None => return Ok(None),
            Some(key) => key,
        };
        let value = cursor.value();

        let key = Uint8Array::new(&key).to_vec();
        let value = value.dyn_into::<Uint8Array>().unwrap().to_vec();

        Ok(Some(IdbEntry { key, value }))
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use futures::StreamExt;
    use indexed_db_futures::prelude::*;
    use indexed_db_futures::web_sys::IdbTransactionMode::*;
    use once_cell::sync::Lazy;
    use wasm_bindgen_futures::spawn_local;
    use wasm_bindgen_test::{console_log, wasm_bindgen_test as test, wasm_bindgen_test_configure};
    use yrs::{Doc, GetString, ReadTxn, Text, Transact};
    use yrs_kvstore_async::{DocOps, KVStore};

    use crate::{IdbError, IdbStore};

    wasm_bindgen_test_configure!(run_in_browser);

    type Result<T, E = IdbError> = std::result::Result<T, E>;

    const OJ_NAME: &str = "test_object_store";

    /// Incrementing the db name, in case we need to avoid deadlocks because of concurrent tests.
    const GET_DB_NAME: Lazy<
        std::sync::Mutex<
            std::cell::RefCell<std::iter::Map<std::ops::RangeFrom<usize>, fn(usize) -> String>>,
        >,
    > = Lazy::new(|| {
        std::sync::Mutex::new(std::cell::RefCell::new(
            (0..).map(|i| format!("test_db_{}", i)),
        ))
    });

    /// Runs a function with an IdbDatabase, and cleans up the database afterwards.
    async fn with_idb_database<'a, F, Fut>(func: F) -> Result<()>
    where
        F: FnOnce(Rc<IdbDatabase>) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let db_name = &{ (GET_DB_NAME.lock().unwrap().borrow_mut()).next().unwrap() };
        let db = Rc::new(IdbStore::prepare_db(db_name, OJ_NAME).await.unwrap());

        func(db).await?;

        // lifetime errors otherwise.
        let db = IdbStore::prepare_db(db_name, OJ_NAME).await.unwrap();

        Ok(db.delete()?.await?)
    }

    #[test]
    async fn test_peek_back() -> Result<()> {
        use crate::IdbEntry;
        with_idb_database(move |db| async move {
            let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
            let object_store = db_txn.object_store(OJ_NAME)?;
            let store = IdbStore::new(object_store);

            store.upsert("a".as_bytes(), "a value".as_bytes()).await?;
            store.upsert("c".as_bytes(), "c value".as_bytes()).await?;
            store.upsert("e".as_bytes(), "e value".as_bytes()).await?;

            db_txn.await.into_result()?;

            let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
            let object_store = db_txn.object_store(OJ_NAME)?;
            let store = IdbStore::new(object_store);

            // when the upper bound key is present
            assert_eq!(
                store.peek_back("e".as_bytes()).await.unwrap(),
                Some(IdbEntry {
                    key: "e".as_bytes().to_vec(),
                    value: "e value".as_bytes().to_vec()
                })
            );

            // when the upper bound key is present, but is not the final key
            assert_eq!(
                store.peek_back("a".as_bytes()).await.unwrap(),
                Some(IdbEntry {
                    key: "a".as_bytes().to_vec(),
                    value: "a value".as_bytes().to_vec()
                })
            );

            // when the upper bound key is not present, and is beyond the final key
            assert_eq!(
                store.peek_back("f".as_bytes()).await.unwrap(),
                Some(IdbEntry {
                    key: "e".as_bytes().to_vec(),
                    value: "e value".as_bytes().to_vec()
                })
            );

            // when the upper bound key is not present, and is between two keys
            assert_eq!(
                store.peek_back("d".as_bytes()).await.unwrap(),
                Some(IdbEntry {
                    key: "c".as_bytes().to_vec(),
                    value: "c value".as_bytes().to_vec()
                })
            );

            Ok(())
        })
        .await
    }

    #[test]
    async fn create_get_remove() -> Result<()> {
        with_idb_database(move |db| async move {
            // insert document
            {
                let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
                let object_store = db_txn.object_store(OJ_NAME)?;
                let store = IdbStore::new(object_store);

                let doc = Doc::new();
                let text = doc.get_or_insert_text("text");
                let mut txn = doc.transact_mut();
                text.insert(&mut txn, 0, "hello");

                store.insert_doc("doc", &txn).await.unwrap();
                db_txn.await.into_result()?;
            }

            // retrieve document
            {
                let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
                let object_store = db_txn.object_store(OJ_NAME)?;
                let store = IdbStore::new(object_store);

                let doc = Doc::new();
                let text = doc.get_or_insert_text("text");
                let (doc, _) = store.load_doc("doc", doc).await.unwrap();

                let txn = doc.transact_mut();
                assert_eq!(text.get_string(&txn), "hello");

                let (sv, completed) = store.get_state_vector("doc").await.unwrap();
                assert_eq!(sv, Some(txn.state_vector()));
                assert!(completed);
            }

            // remove document
            {
                let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
                let object_store = db_txn.object_store(OJ_NAME)?;
                let store = IdbStore::new(object_store);

                store.clear_doc("doc").await.unwrap();

                let doc = Doc::new();
                let text = doc.get_or_insert_text("text");
                let (doc, _) = store.load_doc("doc", doc).await.unwrap();

                let txn = doc.transact_mut();
                assert_eq!(text.get_string(&txn), "");

                let (sv, completed) = DocOps::get_state_vector(&store, "doc").await.unwrap();
                assert!(sv.is_none());
                assert!(completed);
            };

            Ok(())
        })
        .await
    }

    #[test]
    async fn multi_insert() -> Result<()> {
        with_idb_database(move |db| async move {
            // insert document twice
            {
                let doc = Doc::new();
                let text = doc.get_or_insert_text("text");
                let mut txn = doc.transact_mut();
                text.push(&mut txn, "hello");

                let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
                let object_store = db_txn.object_store(OJ_NAME)?;
                let store = IdbStore::new(object_store);

                store.insert_doc("doc", &txn).await.unwrap();

                text.push(&mut txn, " world");

                store.insert_doc("doc", &txn).await.unwrap();

                db_txn.await.into_result()?;
            }

            // retrieve document
            {
                let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
                let object_store = db_txn.object_store(OJ_NAME)?;
                let store = IdbStore::new(object_store);

                let doc = Doc::new();
                let text = doc.get_or_insert_text("text");
                let (doc, _) = store.load_doc("doc", doc).await.unwrap();

                let txn = doc.transact();

                assert_eq!(text.get_string(&txn), "hello world");
            }

            Ok(())
        })
        .await
    }

    #[test]
    async fn incremental_updates() -> Result<()> {
        const DOC_NAME: &str = "doc";

        with_idb_database(move |db| async move {
            // store document updates
            {
                let doc = Doc::new();
                let text = doc.get_or_insert_text("text");

                let db2 = db.clone();
                let db3 = db.clone();
                let _sub = doc.observe_update_v1(move |_, u| {
                    let update = u.update.clone();
                    let db2 = db2.clone();
                    spawn_local(async move {
                        let db_txn = db2
                            .transaction_on_one_with_mode(OJ_NAME, Readwrite)
                            .unwrap();
                        let object_store = db_txn.object_store(OJ_NAME).unwrap();
                        let store = IdbStore::new(object_store);
                        store.push_update(DOC_NAME, &update).await.unwrap();
                        store.flush_doc(DOC_NAME).await.unwrap();
                        db_txn.await.into_result().unwrap();
                    })
                });

                let mut txn = doc.transact_mut();
                // generate 3 updates
                text.push(&mut txn, "a");
                text.push(&mut txn, "b");
                text.push(&mut txn, "c");

                {
                    let db3 = db3.clone();
                    let db_txn = db3.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
                    let object_store = db_txn.object_store(OJ_NAME)?;
                    let store = IdbStore::new(object_store);

                    let doc = Doc::new();
                    let (doc, _) = store.load_doc(DOC_NAME, doc).await.unwrap();
                    let text = doc.get_or_insert_text("text");

                    console_log!("text: {}", text.get_string(&doc.transact()));
                }

                txn.commit();
            }

            // load document
            {
                let doc = Doc::new();

                let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
                let object_store = db_txn.object_store(OJ_NAME)?;
                let store = IdbStore::new(object_store);

                let (doc, _) = store.load_doc(DOC_NAME, doc).await.unwrap();
                let text = doc.get_or_insert_text("text");
                let txn = doc.transact_mut();

                assert_eq!(text.get_string(&txn), "abc");
            }

            // flush document
            {
                let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
                let object_store = db_txn.object_store(OJ_NAME)?;
                let store = IdbStore::new(object_store);
                let doc = store.flush_doc(DOC_NAME).await.unwrap().unwrap();
                db_txn.await.into_result()?;

                let text = doc.get_or_insert_text("text");

                assert_eq!(text.get_string(&doc.transact()), "abc");
            }

            Ok(())
        })
        .await
    }

    #[test]
    async fn state_vector_updates_only() -> Result<()> {
        with_idb_database(move |db| async move {
            const DOC_NAME: &str = "doc";

            // store document updates
            {
                let doc = Doc::new();
                let text = doc.get_or_insert_text("text");

                let db2 = db.clone();
                let _sub = doc.observe_update_v1(move |_, u| {
                    let update = u.update.clone();
                    let db2 = db2.clone();
                    spawn_local(async move {
                        let db_txn = db2
                            .transaction_on_one_with_mode(OJ_NAME, Readwrite)
                            .unwrap();
                        let object_store = db_txn.object_store(OJ_NAME).unwrap();
                        let store = IdbStore::new(object_store);

                        store.push_update(DOC_NAME, &update).await.unwrap();
                        db_txn.await.into_result().unwrap();
                    })
                });
                // generate 3 updates
                text.push(&mut doc.transact_mut(), "a");
                text.push(&mut doc.transact_mut(), "b");
                text.push(&mut doc.transact_mut(), "c");

                let sv = doc.transact().state_vector();
                sv
            };

            let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
            let object_store = db_txn.object_store(OJ_NAME)?;
            let store = IdbStore::new(object_store);

            let (sv, completed) = store.get_state_vector(DOC_NAME).await.unwrap();
            assert!(sv.is_none());
            assert!(!completed); // since it's not completed, we should recalculate state vector from doc state

            Ok(())
        })
        .await
    }

    #[test]
    async fn state_diff_from_updates() -> Result<()> {
        const DOC_NAME: &str = "doc";
        with_idb_database(move |db| async move {
            let (sv, expected) = {
                let doc = Doc::new();
                let text = doc.get_or_insert_text("text");

                let db2 = db.clone();
                let _sub = doc.observe_update_v1(move |_, u| {
                    let update = u.update.clone();
                    let db2 = db2.clone();
                    spawn_local(async move {
                        let db_txn = db2
                            .transaction_on_one_with_mode(OJ_NAME, Readwrite)
                            .unwrap();
                        let object_store = db_txn.object_store(OJ_NAME).unwrap();
                        let store = IdbStore::new(object_store);
                        store.push_update(DOC_NAME, &update).await.unwrap();
                        db_txn.await.into_result().unwrap();
                    })
                });

                // generate 3 updates
                text.push(&mut doc.transact_mut(), "a");
                text.push(&mut doc.transact_mut(), "b");
                let sv = doc.transact().state_vector();
                text.push(&mut doc.transact_mut(), "c");
                let update = doc.transact().encode_diff_v1(&sv);
                (sv, update)
            };

            let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
            let object_store = db_txn.object_store(OJ_NAME)?;
            let store = IdbStore::new(object_store);
            let actual = store.get_diff(DOC_NAME, &sv).await.unwrap();
            assert_eq!(actual, Some(expected));

            Ok(())
        })
        .await
    }

    #[test]
    async fn state_diff_from_doc() -> Result<()> {
        const DOC_NAME: &str = "doc";

        with_idb_database(move |db| async move {
            let (sv, expected) = {
                let doc = Doc::new();
                let text = doc.get_or_insert_text("text");
                // generate 3 updates
                text.push(&mut doc.transact_mut(), "a");
                text.push(&mut doc.transact_mut(), "b");
                let sv = doc.transact().state_vector();
                text.push(&mut doc.transact_mut(), "c");
                let update = doc.transact().encode_diff_v1(&sv);

                let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
                let object_store = db_txn.object_store(OJ_NAME)?;
                let store = IdbStore::new(object_store);
                store.insert_doc(DOC_NAME, &doc.transact()).await.unwrap();
                db_txn.await.into_result()?;

                (sv, update)
            };

            let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
            let object_store = db_txn.object_store(OJ_NAME)?;
            let store = IdbStore::new(object_store);
            let actual = store.get_diff(DOC_NAME, &sv).await.unwrap();
            assert_eq!(actual, Some(expected));

            Ok(())
        })
        .await
    }

    #[test]
    async fn doc_meta() -> Result<()> {
        const DOC_NAME: &str = "doc";

        with_idb_database(move |db| async move {
            let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
            let object_store = db_txn.object_store(OJ_NAME)?;
            let store = IdbStore::new(object_store);
            let value = store.get_meta(DOC_NAME, "key").await.unwrap();
            assert!(value.is_none());
            store
                .insert_meta(DOC_NAME, "key", "value1".as_bytes())
                .await
                .unwrap();
            db_txn.await.into_result()?;

            let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
            let object_store = db_txn.object_store(OJ_NAME)?;
            let store = IdbStore::new(object_store);

            let prev = store
                .get_meta(DOC_NAME, "key")
                .await
                .unwrap()
                .map(Vec::from);
            store
                .insert_meta(DOC_NAME, "key", "value2".as_bytes())
                .await
                .unwrap();
            db_txn.await.into_result()?;
            assert_eq!(prev.as_deref(), Some("value1".as_bytes()));

            let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
            let object_store = db_txn.object_store(OJ_NAME)?;
            let store = IdbStore::new(object_store);
            let prev = store
                .get_meta(DOC_NAME, "key")
                .await
                .unwrap()
                .map(Vec::from);
            store.remove_meta(DOC_NAME, "key").await.unwrap();
            assert_eq!(prev.as_deref(), Some("value2".as_bytes()));
            let value = store.get_meta(DOC_NAME, "key").await.unwrap();
            assert!(value.is_none());

            Ok(())
        })
        .await
    }

    #[test]
    async fn doc_meta_iter() -> Result<()> {
        with_idb_database(move |db| async move {
            let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
            let object_store = db_txn.object_store(OJ_NAME)?;
            let store = IdbStore::new(object_store);

            store.insert_meta("A", "key1", [1].as_ref()).await.unwrap();
            store.insert_meta("B", "key2", [2].as_ref()).await.unwrap();
            store.insert_meta("B", "key3", [3].as_ref()).await.unwrap();
            store.insert_meta("C", "key4", [4].as_ref()).await.unwrap();

            let mut i = store.iter_meta("B").await.unwrap();
            assert_eq!(i.next().await, Some(("key2".as_bytes().into(), [2].into())));
            assert_eq!(i.next().await, Some(("key3".as_bytes().into(), [3].into())));
            assert!(i.next().await.is_none());

            Ok(())
        })
        .await
    }

    #[test]
    async fn doc_iter() -> Result<()> {
        with_idb_database(move |db| async move {
            // insert metadata
            {
                let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
                let object_store = db_txn.object_store(OJ_NAME)?;
                let store = IdbStore::new(object_store);

                store.insert_meta("A", "key1", [1].as_ref()).await.unwrap();
                db_txn.await.into_result()?;
            }

            // insert full doc state
            {
                let doc = Doc::new();
                let text = doc.get_or_insert_text("text");
                let mut txn = doc.transact_mut();
                text.push(&mut txn, "hello world");

                let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
                let object_store = db_txn.object_store(OJ_NAME)?;
                let store = IdbStore::new(object_store);

                store.insert_doc("B", &txn).await.unwrap();
                db_txn.await.into_result()?;
            }

            // insert update
            {
                let doc = Doc::new();

                let db2 = db.clone();
                let _sub = doc.observe_update_v1(move |_, u| {
                    let db2 = db2.clone();
                    let update = u.update.clone();
                    spawn_local(async move {
                        let db_txn = db2
                            .transaction_on_one_with_mode(OJ_NAME, Readwrite)
                            .unwrap();
                        let object_store = db_txn.object_store(OJ_NAME).unwrap();
                        let store = IdbStore::new(object_store);

                        store.push_update("C", &update).await.unwrap();
                        db_txn.await.into_result().unwrap();
                    })
                });
                let text = doc.get_or_insert_text("text");
                let mut txn = doc.transact_mut();
                text.push(&mut txn, "hello world");
            }

            {
                let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
                let object_store = db_txn.object_store(OJ_NAME)?;
                let store = IdbStore::new(object_store);

                let mut i = store.iter_docs().await.unwrap();
                assert_eq!(i.next().await, Some("A".as_bytes().into()));
                assert_eq!(i.next().await, Some("B".as_bytes().into()));
                assert_eq!(i.next().await, Some("C".as_bytes().into()));
                assert!(i.next().await.is_none());
            }

            // clear doc
            {
                let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
                let object_store = db_txn.object_store(OJ_NAME)?;
                let store = IdbStore::new(object_store);

                store.clear_doc("B").await.unwrap();
                db_txn.await.into_result()?;
            }

            {
                let db_txn = db.transaction_on_one_with_mode(OJ_NAME, Readwrite)?;
                let object_store = db_txn.object_store(OJ_NAME)?;
                let store = IdbStore::new(object_store);

                let mut i = store.iter_docs().await.unwrap();
                assert_eq!(i.next().await, Some("A".as_bytes().into()));
                assert_eq!(i.next().await, Some("C".as_bytes().into()));
                assert!(i.next().await.is_none());
            }

            Ok(())
        })
        .await
    }
}
