use crate::{sql::{engine::Transaction, execution::{Executor, ResultSet}}, error::Result};

pub struct DropTable {
    table: String,
}

impl DropTable {
    pub fn new(table: String) -> Box<Self> {
        Box::new(Self { table })
    }
}

impl<T: Transaction> Executor<T> for DropTable {
    fn execute(self: Box<Self>, txn: &mut T) -> Result<ResultSet> {
        txn.delete_table(&self.table)?;
        Ok(ResultSet::DropTable { name: self.table })
    }
}