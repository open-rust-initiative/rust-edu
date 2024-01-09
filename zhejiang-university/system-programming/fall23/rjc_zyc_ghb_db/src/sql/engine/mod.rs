pub mod bitcask;

use std::collections::HashSet;

use crate::error::Result;

use super::{schema::catalog::Catalog, types::{Row, Value, expression::Expression}, session::Session};


pub trait Engine: Clone {
    type Transaction: Transaction;

    fn begin(&self) -> Result<Self::Transaction>;

    fn begin_read_only(&self) -> Result<Self::Transaction>;

    fn begin_as_of(&self, version: u64) -> Result<Self::Transaction>;

    fn session(&self) -> Result<Session<Self>> {
        Ok(Session { engine: self.clone(), txn: None })
    }
}

pub trait Transaction: Catalog {
    fn version(&self) -> u64;
    fn read_only(&self) -> bool;

    fn commit(self) -> Result<()>;
    fn rollback(self) -> Result<()>;

    fn create(&mut self, table: &str, row: Row) -> Result<()>;
    fn delete(&mut self, table: &str, id: &Value) -> Result<()>;
    fn read(&self, table: &str, id: &Value) -> Result<Option<Row>>;
    fn scan(&self, table: &str, filter: Option<Expression>) -> Result<Scan>;
    fn update(&mut self, table: &str, id: &Value, row: Row) -> Result<()>;
}

pub type Scan = Box<dyn DoubleEndedIterator<Item = Result<Row>> + Send>;