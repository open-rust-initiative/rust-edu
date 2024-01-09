use std::collections::HashSet;

use crate::{sql::{engine::Transaction, execution::{Executor, ResultSet}, types::expression::Expression}, error::{Result, Error}};

pub struct Update<T: Transaction> {
    table: String,
    source: Box<dyn Executor<T>>,
    expressions: Vec<(usize, Expression)>,
}

impl<T: Transaction> Update<T> {
    pub fn new(
        table: String,
        source: Box<dyn Executor<T>>,
        expressions: Vec<(usize, Expression)>,
    ) -> Box<Self> {
        Box::new(Self { table, source, expressions })
    }
}

impl<T: Transaction> Executor<T> for Update<T> {
    fn execute(self: Box<Self>, txn: &mut T) -> Result<ResultSet> {
        match self.source.execute(txn)? {
            ResultSet::Query { mut rows, .. } => {
                let table = txn.must_read_table(&self.table)?;
                let mut updated = HashSet::new();
                while let Some(row) = rows.next().transpose()? {
                    let id = table.get_row_key(&row)?;
                    if updated.contains(&id) {
                        continue;
                    }
                    let mut new = row.clone();
                    for (field, expr) in &self.expressions {
                        new[*field] = expr.evaluate(Some(&row))?;
                    }
                    txn.update(&table.name, &id, new)?;
                    updated.insert(id);
                }
                Ok(ResultSet::Update { count: updated.len() as u64 })
            }
            r => Err(Error::Internal(format!("Unexpected response {:?}", r))),
        }
    }
}