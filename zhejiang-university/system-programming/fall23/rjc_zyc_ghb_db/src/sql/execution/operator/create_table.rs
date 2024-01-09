use crate::{sql::{schema::table::Table, engine::Transaction, execution::{Executor, ResultSet}}, error::Result};

pub struct CreateTable {
    table: Table,
}

impl CreateTable {
    pub fn new(table: Table) -> Box<Self> {
        Box::new(Self { table })
    }
}

impl<T: Transaction> Executor<T> for CreateTable {
    fn execute(self: Box<Self>, txn: &mut T) -> Result<ResultSet> {
        let name = self.table.name.clone();
        txn.create_table(self.table)?;
        Ok(ResultSet::CreateTable { name })
    }
}