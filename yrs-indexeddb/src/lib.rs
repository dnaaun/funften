use std::cell::RefCell;
use std::fmt::Display;
use std::task::Poll;

use futures::future::OptionFuture;
use futures::FutureExt;
use indexed_db_futures::prelude::*;
use indexed_db_futures::web_sys::wasm_bindgen::JsValue;
use indexed_db_futures::web_sys::IdbKeyRange;
use indexed_db_futures::{
    request::OpenDbRequest, web_sys::DomException, IdbDatabase, IdbVersionChangeEvent,
};
use js_sys::wasm_bindgen::JsCast;
use js_sys::{ArrayBuffer, Object, Uint8Array};
use wasm_bindgen_test::console_log;
use yrs_kvstore_async::{KVEntry, KVStore};

struct IdbStore<'a> {
    object_store: IdbObjectStore<'a>,
}

impl<'a> IdbStore<'a> {
    fn new(object_store: IdbObjectStore<'a>) -> Self {
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

struct IdbStream<'a> {
    cursor: Option<IdbCursorWithValue<'a, IdbObjectStore<'a>>>,
}

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
    use std::{rc::Rc, time::Duration};

    use indexed_db_futures::prelude::*;
    use indexed_db_futures::web_sys::IdbTransactionMode::*;
    use js_sys::JsString;
    use once_cell::sync::Lazy;
    use tap::TapFallible;
    use wasm_bindgen_futures::spawn_local;
    use wasm_bindgen_test::{console_log, wasm_bindgen_test as test, wasm_bindgen_test_configure};
    use yrs::{Doc, GetString, ReadTxn, Text, Transact};
    use yrs_kvstore_async::DocOps;

    use crate::{IdbError, IdbStore};

    wasm_bindgen_test_configure!(run_in_browser);

    type IResult<T, E = IdbError> = Result<T, E>;

    const OJ_NAME: &str = "test_object_store";

    /// We need to increment the db name to avoid deadlocks.
    const GET_DB_NAME: Lazy<
        std::sync::Mutex<
            std::cell::RefCell<std::iter::Map<std::ops::RangeFrom<usize>, fn(usize) -> String>>,
        >,
    > = Lazy::new(|| {
        std::sync::Mutex::new(std::cell::RefCell::new(
            (0..).map(|i| format!("test_db_{}", i)),
        ))
    });

    async fn with_idb<'a, F, Fut>(func: F) -> IResult<()>
    where
        F: FnOnce(Rc<IdbDatabase>) -> Fut,
        Fut: std::future::Future<Output = IResult<()>>,
    {
        let db_name = &{ (GET_DB_NAME.lock().unwrap().borrow_mut()).next().unwrap() };
        let db = Rc::new(IdbStore::prepare_db(db_name, OJ_NAME).await.unwrap());

        func(db).await?;

        // lifetime errors otherwise.
        let db = IdbStore::prepare_db(db_name, OJ_NAME).await.unwrap();

        Ok(db.delete()?.await?)
    }

    #[test]
    async fn create_get_remove() -> IResult<()> {
        with_idb(move |db| async move {
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
    async fn multi_insert() -> IResult<()> {
        with_idb(move |db| async move {
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
    async fn incremental_updates() -> IResult<()> {
        const DOC_NAME: &str = "doc";

        with_idb(move |db| async move {
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

    // #[test]
    // fn state_vector_updates_only() {
    //     const DOC_NAME: &str = "doc";
    //     let cleaner = Cleaner::new("lmdb-state_vector_updates_only");
    //     let env = init_env(cleaner.dir());
    //     let h = env.create_db("yrs", DbCreate).unwrap();
    //     let env = Arc::new(env);
    //     let h = Arc::new(h);

    //     // store document updates
    //     {
    //         let doc = Doc::new();
    //         let text = doc.get_or_insert_text("text");
    //         let env = env.clone();
    //         let h = h.clone();
    //         let _sub = doc.observe_update_v1(move |_, u| {
    //             let db_txn = env.new_transaction().unwrap();
    //             let db = LmdbStore::from(db_txn.bind(&h));
    //             db.push_update(DOC_NAME, &u.update).unwrap();
    //             db_txn.commit().unwrap();
    //         });
    //         // generate 3 updates
    //         text.push(&mut doc.transact_mut(), "a");
    //         text.push(&mut doc.transact_mut(), "b");
    //         text.push(&mut doc.transact_mut(), "c");

    //         let sv = doc.transact().state_vector();
    //         sv
    //     };

    //     let db_txn = env.get_reader().unwrap();
    //     let db = LmdbStore::from(db_txn.bind(&h));
    //     let (sv, completed) = db.get_state_vector(DOC_NAME).unwrap();
    //     assert!(sv.is_none());
    //     assert!(!completed); // since it's not completed, we should recalculate state vector from doc state
    // }

    // #[test]
    // fn state_diff_from_updates() {
    //     const DOC_NAME: &str = "doc";
    //     let cleaner = Cleaner::new("lmdb-state_diff_from_updates");
    //     let env = init_env(cleaner.dir());
    //     let h = env.create_db("yrs", DbCreate).unwrap();
    //     let env = Arc::new(env);
    //     let h = Arc::new(h);

    //     let (sv, expected) = {
    //         let doc = Doc::new();
    //         let text = doc.get_or_insert_text("text");

    //         let env = env.clone();
    //         let h = h.clone();
    //         let _sub = doc.observe_update_v1(move |_, u| {
    //             let db_txn = env.new_transaction().unwrap();
    //             let db = LmdbStore::from(db_txn.bind(&h));
    //             db.push_update(DOC_NAME, &u.update).unwrap();
    //             db_txn.commit().unwrap();
    //         });

    //         // generate 3 updates
    //         text.push(&mut doc.transact_mut(), "a");
    //         text.push(&mut doc.transact_mut(), "b");
    //         let sv = doc.transact().state_vector();
    //         text.push(&mut doc.transact_mut(), "c");
    //         let update = doc.transact().encode_diff_v1(&sv);
    //         (sv, update)
    //     };

    //     let db_txn = env.get_reader().unwrap();
    //     let db = LmdbStore::from(db_txn.bind(&h));
    //     let actual = db.get_diff(DOC_NAME, &sv).unwrap();
    //     assert_eq!(actual, Some(expected));
    // }

    // #[test]
    // fn state_diff_from_doc() {
    //     const DOC_NAME: &str = "doc";
    //     let cleaner = Cleaner::new("lmdb-state_diff_from_doc");
    //     let env = init_env(cleaner.dir());
    //     let h = env.create_db("yrs", DbCreate).unwrap();

    //     let (sv, expected) = {
    //         let doc = Doc::new();
    //         let text = doc.get_or_insert_text("text");
    //         // generate 3 updates
    //         text.push(&mut doc.transact_mut(), "a");
    //         text.push(&mut doc.transact_mut(), "b");
    //         let sv = doc.transact().state_vector();
    //         text.push(&mut doc.transact_mut(), "c");
    //         let update = doc.transact().encode_diff_v1(&sv);

    //         let db_txn = env.new_transaction().unwrap();
    //         let db = LmdbStore::from(db_txn.bind(&h));
    //         db.insert_doc(DOC_NAME, &doc.transact()).unwrap();
    //         db_txn.commit().unwrap();

    //         (sv, update)
    //     };

    //     let db_txn = env.get_reader().unwrap();
    //     let db = LmdbStore::from(db_txn.bind(&h));
    //     let actual = db.get_diff(DOC_NAME, &sv).unwrap();
    //     assert_eq!(actual, Some(expected));
    // }

    // #[test]
    // fn doc_meta() {
    //     const DOC_NAME: &str = "doc";
    //     let cleaner = Cleaner::new("lmdb-doc_meta");
    //     let env = init_env(cleaner.dir());
    //     let h = env.create_db("yrs", DbCreate).unwrap();

    //     let db_txn = env.new_transaction().unwrap();
    //     let db = LmdbStore::from(db_txn.bind(&h));
    //     let value = db.get_meta(DOC_NAME, "key").unwrap();
    //     assert!(value.is_none());
    //     db.insert_meta(DOC_NAME, "key", "value1".as_bytes())
    //         .unwrap();
    //     db_txn.commit().unwrap();

    //     let db_txn = env.new_transaction().unwrap();
    //     let db = LmdbStore::from(db_txn.bind(&h));
    //     let prev = db.get_meta(DOC_NAME, "key").unwrap().map(Vec::from);
    //     db.insert_meta(DOC_NAME, "key", "value2".as_bytes())
    //         .unwrap();
    //     db_txn.commit().unwrap();
    //     assert_eq!(prev.as_deref(), Some("value1".as_bytes()));

    //     let db_txn = env.new_transaction().unwrap();
    //     let db = LmdbStore::from(db_txn.bind(&h));
    //     let prev = db.get_meta(DOC_NAME, "key").unwrap().map(Vec::from);
    //     db.remove_meta(DOC_NAME, "key").unwrap();
    //     assert_eq!(prev.as_deref(), Some("value2".as_bytes()));
    //     let value = db.get_meta(DOC_NAME, "key").unwrap();
    //     assert!(value.is_none());
    // }

    // #[test]
    // fn doc_meta_iter() {
    //     let cleaner = Cleaner::new("lmdb-doc_meta_iter");
    //     let env = init_env(cleaner.dir());
    //     let h = env.create_db("yrs", DbCreate).unwrap();
    //     let db_txn = env.new_transaction().unwrap();
    //     let db = LmdbStore::from(db_txn.bind(&h));

    //     db.insert_meta("A", "key1", [1].as_ref()).unwrap();
    //     db.insert_meta("B", "key2", [2].as_ref()).unwrap();
    //     db.insert_meta("B", "key3", [3].as_ref()).unwrap();
    //     db.insert_meta("C", "key4", [4].as_ref()).unwrap();

    //     let mut i = db.iter_meta("B").unwrap();
    //     assert_eq!(i.next(), Some(("key2".as_bytes().into(), [2].into())));
    //     assert_eq!(i.next(), Some(("key3".as_bytes().into(), [3].into())));
    //     assert!(i.next().is_none());
    // }

    // #[test]
    // fn doc_iter() {
    //     let cleaner = Cleaner::new("lmdb-doc_iter");
    //     let env = init_env(cleaner.dir());
    //     let h = env.create_db("yrs", DbCreate).unwrap();
    //     let env = Arc::new(env);
    //     let h = Arc::new(h);

    //     // insert metadata
    //     {
    //         let db_txn = env.new_transaction().unwrap();
    //         let db = LmdbStore::from(db_txn.bind(&h));
    //         db.insert_meta("A", "key1", [1].as_ref()).unwrap();
    //         db_txn.commit().unwrap();
    //     }

    //     // insert full doc state
    //     {
    //         let doc = Doc::new();
    //         let text = doc.get_or_insert_text("text");
    //         let mut txn = doc.transact_mut();
    //         text.push(&mut txn, "hello world");
    //         let db_txn = env.new_transaction().unwrap();
    //         let db = LmdbStore::from(db_txn.bind(&h));
    //         db.insert_doc("B", &txn).unwrap();
    //         db_txn.commit().unwrap();
    //     }

    //     // insert update
    //     {
    //         let doc = Doc::new();
    //         let env = env.clone();
    //         let h = h.clone();
    //         let _sub = doc.observe_update_v1(move |_, u| {
    //             let db_txn = env.new_transaction().unwrap();
    //             let db = LmdbStore::from(db_txn.bind(&h));
    //             db.push_update("C", &u.update).unwrap();
    //             db_txn.commit().unwrap();
    //         });
    //         let text = doc.get_or_insert_text("text");
    //         let mut txn = doc.transact_mut();
    //         text.push(&mut txn, "hello world");
    //     }

    //     {
    //         let db_txn = env.get_reader().unwrap();
    //         let db = LmdbStore::from(db_txn.bind(&h));
    //         let mut i = db.iter_docs().unwrap();
    //         assert_eq!(i.next(), Some("A".as_bytes().into()));
    //         assert_eq!(i.next(), Some("B".as_bytes().into()));
    //         assert_eq!(i.next(), Some("C".as_bytes().into()));
    //         assert!(i.next().is_none());
    //     }

    //     // clear doc
    //     {
    //         let db_txn = env.new_transaction().unwrap();
    //         let db = LmdbStore::from(db_txn.bind(&h));
    //         db.clear_doc("B").unwrap();
    //         db_txn.commit().unwrap();
    //     }

    //     {
    //         let db_txn = env.get_reader().unwrap();
    //         let db = LmdbStore::from(db_txn.bind(&h));
    //         let mut i = db.iter_docs().unwrap();
    //         assert_eq!(i.next(), Some("A".as_bytes().into()));
    //         assert_eq!(i.next(), Some("C".as_bytes().into()));
    //         assert!(i.next().is_none());
    //     }
    // }
}
