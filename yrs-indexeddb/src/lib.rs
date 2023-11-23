use std::fmt::Display;
use std::task::Poll;

use indexed_db_futures::prelude::*;
use indexed_db_futures::web_sys::wasm_bindgen::JsValue;
use indexed_db_futures::web_sys::IdbKeyRange;
use indexed_db_futures::{
    request::OpenDbRequest, web_sys::DomException, IdbDatabase, IdbVersionChangeEvent,
};
use js_sys::wasm_bindgen::JsCast;
use js_sys::{Object, Uint8Array};
use yrs_kvstore_async::{KVEntry, KVStore};

struct IdbStore {
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

struct IdbStream<'a> {
    txn: IdbTransaction<'a>,
    object_store_name: &'a str,
    range: IdbKeyRange,
    object_store: Option<IdbObjectStore<'a>>,
    cursor: Option<Option<IdbCursorWithValue<'a, IdbObjectStore<'a>>>>,
}

impl<'a> IdbStream<'a> {
    async fn setup(&'a mut self) -> Result<(), IdbError> {
        self.object_store = Some(self.txn.object_store(self.object_store_name)?);
        self.cursor = Some(
            self.object_store
                .as_ref()
                .unwrap()
                .open_cursor_with_range(&self.range)?
                .await?,
        );

        Ok(())
    }
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
                    Err(e) => "InvalidValueInIdb: could not convert error to Object".to_owned(),
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
        let cursor = self.cursor.as_ref().unwrap();
        let object_store = self.object_store.as_ref().unwrap();

        let cursor = match cursor {
            None => return Poll::Ready(None),
            Some(cursor) => cursor,
        };

        let key = match cursor.key() {
            None => return Poll::Ready(None),
            Some(key) => key,
        };

        let value = cursor.value();

        let key = key.dyn_into::<Uint8Array>().unwrap().to_vec();
        let value = value.dyn_into::<Uint8Array>().unwrap().to_vec();

        Poll::Ready(Some(IdbEntry { key, value }))
    }
}

impl<'a> KVStore<'a> for IdbStore {
    type Error = IdbError;

    type Cursor = IdbStream<'a>;

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

    async fn iter_range<'b>(
        &'a self,
        from: &'b [u8],
        to: &'b [u8],
    ) -> Result<IdbStream<'a>, Self::Error> {
        let from = js_sys::Uint8Array::from(from)
            .dyn_into::<JsValue>()
            .unwrap();
        let to = js_sys::Uint8Array::from(to).dyn_into::<JsValue>().unwrap();

        let txn: IdbTransaction<'a> = self.db.transaction_on_one(&self.object_store_name)?;

        let range = IdbKeyRange::bound(&from, &to).unwrap();

        let mut stream_builder = IdbStream {
            txn,
            range,
            object_store_name: self.object_store_name.as_str(),
            object_store: None,
            cursor: None,
        };

        Ok(stream_builder)
    }

    async fn peek_back(&self, key: &[u8]) -> Result<Option<Self::Entry>, Self::Error> {
        let txn = self.db.transaction_on_one(&self.object_store_name)?;
        let object_store = txn.object_store(&self.object_store_name)?;

        let to = js_sys::Uint8Array::from(key);
        let range = IdbKeyRange::upper_bound(&to.dyn_into::<JsValue>().unwrap()).unwrap();
        let cursor = object_store
            .open_cursor_with_range_and_direction(&range, IdbCursorDirection::Prev)?
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

        let key = key.dyn_into::<Uint8Array>().unwrap().to_vec();
        let value = value.dyn_into::<Uint8Array>().unwrap().to_vec();

        Ok(Some(IdbEntry { key, value }))
    }
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

    use crate::IdbStore;

    wasm_bindgen_test_configure!(run_in_browser);

    #[test]
    async fn plain() {
        let store = IdbStore::new("test".to_owned(), "test".to_owned()).await.unwrap();
    }

    // #[test]
    // fn create_get_remove() {
    //     let cleaner = Cleaner::new("lmdb-create_get_remove");
    //     let env = init_env(cleaner.dir());
    //     let h = env.create_db("yrs", DbCreate).unwrap();

    //     // insert document
    //     {
    //         let doc = Doc::new();
    //         let text = doc.get_or_insert_text("text");
    //         let mut txn = doc.transact_mut();
    //         text.insert(&mut txn, 0, "hello");

    //         let db_txn = env.new_transaction().unwrap();
    //         let db = LmdbStore::from(db_txn.bind(&h));
    //         db.insert_doc("doc", &txn).unwrap();
    //         db_txn.commit().unwrap();
    //     }

    //     // retrieve document
    //     {
    //         let doc = Doc::new();
    //         let text = doc.get_or_insert_text("text");
    //         let mut txn = doc.transact_mut();
    //         let db_txn = env.get_reader().unwrap();
    //         let db = LmdbStore::from(db_txn.bind(&h));
    //         db.load_doc("doc", &mut txn).unwrap();

    //         assert_eq!(text.get_string(&txn), "hello");

    //         let (sv, completed) = db.get_state_vector("doc").unwrap();
    //         assert_eq!(sv, Some(txn.state_vector()));
    //         assert!(completed);
    //     }

    //     // remove document
    //     {
    //         let db_txn = env.new_transaction().unwrap();
    //         let db = LmdbStore::from(db_txn.bind(&h));

    //         db.clear_doc("doc").unwrap();

    //         let doc = Doc::new();
    //         let text = doc.get_or_insert_text("text");
    //         let mut txn = doc.transact_mut();
    //         db.load_doc("doc", &mut txn).unwrap();

    //         assert_eq!(text.get_string(&txn), "");

    //         let (sv, completed) = db.get_state_vector("doc").unwrap();
    //         assert!(sv.is_none());
    //         assert!(completed);
    //     }
    // }
    // #[test]
    // fn multi_insert() {
    //     let cleaner = Cleaner::new("lmdb-multi_insert");
    //     let env = init_env(cleaner.dir());
    //     let h = env.create_db("yrs", DbCreate).unwrap();

    //     // insert document twice
    //     {
    //         let doc = Doc::new();
    //         let text = doc.get_or_insert_text("text");
    //         let mut txn = doc.transact_mut();
    //         text.push(&mut txn, "hello");

    //         let db_txn = env.new_transaction().unwrap();
    //         let db = LmdbStore::from(db_txn.bind(&h));

    //         db.insert_doc("doc", &txn).unwrap();

    //         text.push(&mut txn, " world");

    //         db.insert_doc("doc", &txn).unwrap();

    //         db_txn.commit().unwrap();
    //     }

    //     // retrieve document
    //     {
    //         let db_txn = env.get_reader().unwrap();
    //         let db = LmdbStore::from(db_txn.bind(&h));

    //         let doc = Doc::new();
    //         let text = doc.get_or_insert_text("text");
    //         let mut txn = doc.transact_mut();
    //         db.load_doc("doc", &mut txn).unwrap();

    //         assert_eq!(text.get_string(&txn), "hello world");
    //     }
    // }

    // #[test]
    // fn incremental_updates() {
    //     const DOC_NAME: &str = "doc";
    //     let cleaner = Cleaner::new("lmdb-incremental_updates");
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
    //     }

    //     // load document
    //     {
    //         let doc = Doc::new();
    //         let text = doc.get_or_insert_text("text");
    //         let mut txn = doc.transact_mut();

    //         let db_txn = env.get_reader().unwrap();
    //         let db = LmdbStore::from(db_txn.bind(&h));
    //         db.load_doc(DOC_NAME, &mut txn).unwrap();

    //         assert_eq!(text.get_string(&txn), "abc");
    //     }

    //     // flush document
    //     {
    //         let db_txn = env.new_transaction().unwrap();
    //         let db = LmdbStore::from(db_txn.bind(&h));
    //         let doc = db.flush_doc(DOC_NAME).unwrap().unwrap();
    //         db_txn.commit().unwrap();

    //         let text = doc.get_or_insert_text("text");

    //         assert_eq!(text.get_string(&doc.transact()), "abc");
    //     }
    // }

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
