use std::sync::{Arc, Mutex};

use crate::{storage, sql::engine::bitcask::KV};

pub struct DBGruad<E: storage::engine::Engine> {
    pub db: Arc<Mutex<KV<E>>>,
}

impl<E: storage::engine::Engine> DBGruad<E> {
    pub fn new(engine: E) -> Self {
        Self { db: Arc::new(Mutex::new(KV::new(engine))) }
    }
}

impl<E: storage::engine::Engine> Clone for DBGruad<E> {
    fn clone(&self) -> Self {
        DBGruad { db: self.db.clone() }
    }
}