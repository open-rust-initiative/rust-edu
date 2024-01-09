use crate::{sql::{engine::Transaction, execution::{Executor, ResultSet}, types::{expression::Expression, Value}}, error::{Error, Result}};

pub struct Filter<T: Transaction> {
    source: Box<dyn Executor<T>>,
    predicate: Expression,
}

impl<T: Transaction> Filter<T> {
    pub fn new(source: Box<dyn Executor<T>>, predicate: Expression) -> Box<Self> {
        Box::new(Self { source, predicate })
    }
}

impl<T: Transaction> Executor<T> for Filter<T> {
    fn execute(self: Box<Self>, txn: &mut T) -> Result<ResultSet> {
        if let ResultSet::Query { columns, rows } = self.source.execute(txn)? {
            let predicate = self.predicate;
            Ok(ResultSet::Query {
                columns,
                rows: Box::new(rows.filter_map(move |r| {
                    r.and_then(|row| match predicate.evaluate(Some(&row))? {
                        Value::Boolean(true) => Ok(Some(row)),
                        Value::Boolean(false) => Ok(None),
                        Value::Null => Ok(None),
                        value => Err(Error::Value(format!(
                            "Filter returned {}, expected boolean",
                            value
                        ))),
                    })
                    .transpose()
                })),
            })
        } else {
            Err(Error::Internal("Unexpected result".into()))
        }
    }
}