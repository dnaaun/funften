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
