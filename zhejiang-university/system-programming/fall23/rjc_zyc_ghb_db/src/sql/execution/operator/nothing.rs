use crate::{sql::{engine::Transaction, execution::{Executor, ResultSet}, types::Row}, error::Result};

pub struct Nothing;

impl Nothing {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl<T: Transaction> Executor<T> for Nothing {
    fn execute(self: Box<Self>, _: &mut T) -> Result<ResultSet> {
        Ok(ResultSet::Query {
            columns: Vec::new(),
            rows: Box::new(std::iter::once(Ok(Row::new()))),
        })
    }
}
