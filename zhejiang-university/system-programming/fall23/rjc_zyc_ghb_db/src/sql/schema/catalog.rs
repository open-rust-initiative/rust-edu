use crate::error::{Error, Result};

use super::table::{Table, Tables};

/// db 的接口
pub trait Catalog {
    fn create_table(&mut self, table: Table) -> Result<()>;

    fn delete_table(&mut self, table: &str) -> Result<()>;

    fn read_table(&self, table: &str) -> Result<Option<Table>>;

    fn scan_tables(&self) -> Result<Tables>;

    fn must_read_table(&self, table: &str) -> Result<Table> {
        self.read_table(table)?
            .ok_or_else(|| Error::Value(format!("Table {} does not exist", table)))
    }

    fn table_references(&self, table: &str, with_self: bool) -> Result<Vec<(String, Vec<String>)>> {
        Ok(self
            .scan_tables()?
            .filter(|t| with_self || t.name != table)
            .map(|t| {
                (
                    t.name,
                    t.columns
                        .iter()
                        .filter(|c| c.references.as_deref() == Some(table))
                        .map(|c| c.name.clone())
                        .collect::<Vec<_>>(),
                )
            })
            .filter(|(_, cs)| !cs.is_empty())
            .collect())
    }
}