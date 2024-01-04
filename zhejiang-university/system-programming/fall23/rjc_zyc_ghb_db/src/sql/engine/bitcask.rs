use super::Transaction as _;
use crate::error::{Error, Result};
use crate::sql::schema::catalog::Catalog;
use crate::sql::schema::table::{Table, Tables};
use crate::sql::types::expression::Expression;
use crate::sql::types::{Value, Row};
use crate::storage::mvcc::mvcc::MVCC;
use crate::storage::{self, bincode, keycode};

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::clone::Clone;

/// SQL engine 基于 MVCC storage 实现
pub struct KV<E: storage::engine::Engine> {
    pub kv: MVCC<E>,
}

impl<E: storage::engine::Engine> Clone for KV<E> {
    fn clone(&self) -> Self {
        KV { kv: self.kv.clone() }
    }
}

/// 实现一个 storage engine 需要实现的 trait
impl<E: storage::engine::Engine> KV<E> {
    pub fn new(engine: E) -> Self {
        Self { kv: MVCC::new(engine) }
    }

    pub fn resume(
        &self,
        state: crate::storage::mvcc::transaction::TransactionState,
    ) -> Result<<Self as super::Engine>::Transaction> {
        Ok(<Self as super::Engine>::Transaction::new(self.kv.resume(state)?))
    }

    pub fn get_metadata(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.kv.get_unversioned(key)
    }

    pub fn set_metadata(&self, key: &[u8], value: Vec<u8>) -> Result<()> {
        self.kv.set_unversioned(key, value)
    }
}

/// 实现 sql engine 需要实现的 trait
impl<E: storage::engine::Engine> super::Engine for KV<E> {
    type Transaction = Transaction<E>;

    fn begin(&self) -> Result<Self::Transaction> {
        Ok(Self::Transaction::new(self.kv.begin()?))
    }

    fn begin_read_only(&self) -> Result<Self::Transaction> {
        Ok(Self::Transaction::new(self.kv.begin_read_only()?))
    }

    fn begin_as_of(&self, version: u64) -> Result<Self::Transaction> {
        Ok(Self::Transaction::new(self.kv.begin_as_of(version)?))
    }
}

/// 序列化 SQL 的元数据
fn serialize<V: Serialize>(value: &V) -> Result<Vec<u8>> {
    bincode::serialize(value)
}

/// 反序列化 SQL 的元数据
fn deserialize<'a, V: Deserialize<'a>>(bytes: &'a [u8]) -> Result<V> {
    bincode::deserialize(bytes)
}

/// SQL 事务
pub struct Transaction<E: storage::engine::Engine> {
    txn: crate::storage::mvcc::transaction::Transaction<E>,
}

impl<E: storage::engine::Engine> Transaction<E> {
    fn new(txn: crate::storage::mvcc::transaction::Transaction<E>) -> Self {
        Self { txn }
    }

    pub(crate) fn state(&self) -> &crate::storage::mvcc::transaction::TransactionState {
        self.txn.state()
    }
}

impl<E: storage::engine::Engine> super::Transaction for Transaction<E> {
    fn version(&self) -> u64 {
        self.txn.version()
    }

    fn read_only(&self) -> bool {
        self.txn.read_only()
    }

    fn commit(self) -> Result<()> {
        self.txn.commit()
    }

    fn rollback(self) -> Result<()> {
        self.txn.rollback()
    }

    fn create(&mut self, table: &str, row: Row) -> Result<()> {
        let table = self.must_read_table(table)?;
        table.validate_row(&row, self)?;
        let id = table.get_row_key(&row)?;
        if self.read(&table.name, &id)?.is_some() {
            return Err(Error::Value(format!(
                "Primary key {} already exists for table {}",
                id, table.name
            )));
        }
        self.txn.set(&Key::Row((&table.name).into(), (&id).into()).encode()?, serialize(&row)?)?;
        Ok(())
    }

    fn delete(&mut self, table: &str, id: &Value) -> Result<()> {
        let table = self.must_read_table(table)?;
        for (t, cs) in self.table_references(&table.name, true)? {
            let t = self.must_read_table(&t)?;
            let cs = cs
                .into_iter()
                .map(|c| Ok((t.get_column_index(&c)?, c)))
                .collect::<Result<Vec<_>>>()?;
            let mut scan = self.scan(&t.name, None)?;
            while let Some(row) = scan.next().transpose()? {
                for (i, c) in &cs {
                    if &row[*i] == id && (table.name != t.name || id != &table.get_row_key(&row)?) {
                        return Err(Error::Value(format!(
                            "Primary key {} is referenced by table {} column {}",
                            id, t.name, c
                        )));
                    }
                }
            }
        }
        self.txn.delete(&Key::Row(table.name.into(), id.into()).encode()?)
    }

    fn read(&self, table: &str, id: &Value) -> Result<Option<Row>> {
        self.txn
            .get(&Key::Row(table.into(), id.into()).encode()?)?
            .map(|v| deserialize(&v))
            .transpose()
    }

    fn scan(&self, table: &str, filter: Option<Expression>) -> Result<super::Scan> {
        let table = self.must_read_table(table)?;
        Ok(Box::new(
            self.txn
                .scan_prefix(&KeyPrefix::Row((&table.name).into()).encode()?)?
                .iter()
                .map(|r| r.and_then(|(_, v)| deserialize(&v)))
                .filter_map(move |r| match r {
                    Ok(row) => match &filter {
                        Some(filter) => match filter.evaluate(Some(&row)) {
                            Ok(Value::Boolean(b)) if b => Some(Ok(row)),
                            Ok(Value::Boolean(_)) | Ok(Value::Null) => None,
                            Ok(v) => Some(Err(Error::Value(format!(
                                "Filter returned {}, expected boolean",
                                v
                            )))),
                            Err(err) => Some(Err(err)),
                        },
                        None => Some(Ok(row)),
                    },
                    err => Some(err),
                })
                .collect::<Vec<_>>()
                .into_iter(),
        ))
    }

    fn update(&mut self, table: &str, id: &Value, row: Row) -> Result<()> {
        let table = self.must_read_table(table)?;
        // If the primary key changes we do a delete and create, otherwise we replace the row
        if id != &table.get_row_key(&row)? {
            self.delete(&table.name, id)?;
            self.create(&table.name, row)?;
            return Ok(());
        }

        table.validate_row(&row, self)?;
        self.txn.set(&Key::Row(table.name.into(), id.into()).encode()?, serialize(&row)?)
    }
}

impl<E: storage::engine::Engine> Catalog for Transaction<E> {
    fn create_table(&mut self, table: Table) -> Result<()> {
        if self.read_table(&table.name)?.is_some() {
            return Err(Error::Value(format!("Table {} already exists", table.name)));
        }
        table.validate(self)?;
        self.txn.set(&Key::Table((&table.name).into()).encode()?, serialize(&table)?)
    }

    fn delete_table(&mut self, table: &str) -> Result<()> {
        let table = self.must_read_table(table)?;
        if let Some((t, cs)) = self.table_references(&table.name, false)?.first() {
            return Err(Error::Value(format!(
                "Table {} is referenced by table {} column {}",
                table.name, t, cs[0]
            )));
        }
        let mut scan = self.scan(&table.name, None)?;
        while let Some(row) = scan.next().transpose()? {
            self.delete(&table.name, &table.get_row_key(&row)?)?
        }
        self.txn.delete(&Key::Table(table.name.into()).encode()?)
    }

    fn read_table(&self, table: &str) -> Result<Option<Table>> {
        self.txn.get(&Key::Table(table.into()).encode()?)?.map(|v| deserialize(&v)).transpose()
    }

    fn scan_tables(&self) -> Result<Tables> {
        Ok(Box::new(
            self.txn
                .scan_prefix(&KeyPrefix::Table.encode()?)?
                .iter()
                .map(|r| r.and_then(|(_, v)| deserialize(&v)))
                .collect::<Result<Vec<_>>>()?
                .into_iter(),
        ))
    }
}

#[derive(Debug, Deserialize, Serialize)]
enum Key<'a> {
    /// 用于 Table 元数据原理
    Table(Cow<'a, str>),
    /// 用于管理 Table 的数据
    Row(Cow<'a, str>, Cow<'a, Value>),
}

impl<'a> Key<'a> {
    fn encode(self) -> Result<Vec<u8>> {
        keycode::serialize(&self)
    }

    fn decode(bytes: &[u8]) -> Result<Self> {
        keycode::deserialize(bytes)
    }
}

#[derive(Debug, Deserialize, Serialize)]
enum KeyPrefix<'a> {
    Table,
    Row(Cow<'a, str>),
}

impl<'a> KeyPrefix<'a> {
    fn encode(self) -> Result<Vec<u8>> {
        keycode::serialize(&self)
    }
}