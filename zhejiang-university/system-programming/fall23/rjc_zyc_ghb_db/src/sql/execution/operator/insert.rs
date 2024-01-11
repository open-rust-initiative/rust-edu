use std::collections::HashMap;

use crate::{sql::{engine::Transaction, types::{expression::Expression, Value, Row}, schema::table::Table, execution::{Executor, ResultSet}}, error::{Result, Error}};

pub struct Insert {
    table: String,
    columns: Vec<String>,
    rows: Vec<Vec<Expression>>,
}

impl Insert {
    pub fn new(table: String, columns: Vec<String>, rows: Vec<Vec<Expression>>) -> Box<Self> {
        Box::new(Self { table, columns, rows })
    }

    // Builds a row from a set of column names and values, padding it with default values.
    pub fn make_row(table: &Table, columns: &[String], values: Vec<Value>) -> Result<Row> {
        if columns.len() != values.len() {
            return Err(Error::Value("Column and value counts do not match".into()));
        }
        let mut inputs = HashMap::new();
        for (c, v) in columns.iter().zip(values.into_iter()) {
            table.get_column(c)?;
            if inputs.insert(c.clone(), v).is_some() {
                return Err(Error::Value(format!("Column {} given multiple times", c)));
            }
        }
        let mut row = Row::new();
        for column in table.columns.iter() {
            if let Some(value) = inputs.get(&column.name) {
                row.push(value.clone())
            } else if let Some(value) = &column.default {
                row.push(value.clone())
            } else {
                return Err(Error::Value(format!("No value given for column {}", column.name)));
            }
        }
        Ok(row)
    }

    /// Pads a row with default values where possible.
    fn pad_row(table: &Table, mut row: Row) -> Result<Row> {
        for column in table.columns.iter().skip(row.len()) {
            if let Some(default) = &column.default {
                row.push(default.clone())
            } else {
                return Err(Error::Value(format!("No default value for column {}", column.name)));
            }
        }
        Ok(row)
    }
}

impl<T: Transaction> Executor<T> for Insert {
    fn execute(self: Box<Self>, txn: &mut T) -> Result<ResultSet> {
        let table = txn.must_read_table(&self.table)?;
        let mut count = 0;
        for expressions in self.rows {
            let mut row =
                expressions.into_iter().map(|expr| expr.evaluate(None)).collect::<Result<_>>()?;
            if self.columns.is_empty() {
                row = Self::pad_row(&table, row)?;
            } else {
                row = Self::make_row(&table, &self.columns, row)?;
            }
            txn.create(&table.name, row)?;
            count += 1;
        }
        Ok(ResultSet::Create { count })
    }
}