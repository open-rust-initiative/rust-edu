use crate::error::{Result, Error};

use crate::sql::engine::Transaction;

use super::parser::{Parser, ast};
use super::plan::Plan;
use super::{engine::Engine, execution::ResultSet};

pub struct Session<E: Engine> {
    pub engine: E,
    pub txn: Option<E::Transaction>,
}

impl<E: Engine + 'static> Session<E> {
    pub fn execute(&mut self, query: &str) -> Result<ResultSet> {
        match Parser::new(query).parse()? {
            ast::Statement::Begin { .. } if self.txn.is_some() => {
                Err(Error::Value("Already in a transaction".into()))
            }
            ast::Statement::Begin { read_only: true, as_of: None } => {
                let txn: <E as Engine>::Transaction = self.engine.begin_read_only()?;
                let result = ResultSet::Begin { version: txn.version(), read_only: true };
                self.txn = Some(txn);
                Ok(result)
            }
            ast::Statement::Begin { read_only: true, as_of: Some(version) } => {
                let txn = self.engine.begin_as_of(version)?;
                let result = ResultSet::Begin { version, read_only: true };
                self.txn = Some(txn);
                Ok(result)
            }
            ast::Statement::Begin { read_only: false, as_of: Some(_) } => {
                Err(Error::Value("Can't start read-write transaction in a given version".into()))
            }
            ast::Statement::Begin { read_only: false, as_of: None } => {
                let txn = self.engine.begin()?;
                let result = ResultSet::Begin { version: txn.version(), read_only: false };
                self.txn = Some(txn);
                Ok(result)
            }
            ast::Statement::Commit | ast::Statement::Rollback if self.txn.is_none() => {
                Err(Error::Value("Not in a transaction".into()))
            }
            ast::Statement::Commit => {
                let txn = self.txn.take().unwrap();
                let version = txn.version();
                txn.commit()?;
                Ok(ResultSet::Commit { version })
            }
            ast::Statement::Rollback => {
                let txn = self.txn.take().unwrap();
                let version = txn.version();
                txn.rollback()?;
                Ok(ResultSet::Rollback { version })
            }
            ast::Statement::Explain(_) => self.read_with_txn(|_txn| {
                unimplemented!()
            }),
            statement if self.txn.is_some() => Plan::build(statement, self.txn.as_mut().unwrap())?
                .optimize(self.txn.as_mut().unwrap())?
                .execute(self.txn.as_mut().unwrap()),
            statement @ ast::Statement::Select { .. } => {
                let mut txn = self.engine.begin_read_only()?;
                let result =
                    Plan::build(statement, &mut txn)?.optimize(&mut txn)?.execute(&mut txn);
                txn.rollback()?;
                result
            }
            statement => {
                let mut txn = self.engine.begin()?;
                match Plan::build(statement, &mut txn)?.optimize(&mut txn)?.execute(&mut txn) {
                    Ok(result) => {
                        txn.commit()?;
                        Ok(result)
                    }
                    Err(error) => {
                        txn.rollback()?;
                        Err(error)
                    }
                }
            }
        }
    }

    pub fn read_with_txn<R, F>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut E::Transaction) -> Result<R>,
    {
        if let Some(ref mut txn) = self.txn {
            return f(txn);
        }
        let mut txn = self.engine.begin_read_only()?;
        let result = f(&mut txn);
        txn.rollback()?;
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::{error::Result, storage::engine::bitcask::Bitcask, sql::engine::{bitcask::KV, Engine}};


    #[test]
    fn test() -> Result<()> {
        let path = tempdir::TempDir::new("waterdb")?.path().join("waterdb");
        let engine = Bitcask::new(path)?;
        let engine = KV::new(engine);
        let mut session = engine.session()?;

        session.execute("BEGIN")?;
        let queries = vec![
            "CREATE TABLE t (id int primary key, name char)",
            "INSERT INTO t VALUES (1, 'haha')",
            "INSERT INTO t VALUES (2, 'nana')",
            "INSERT INTO t VALUES (3, 'gaga')",
            "UPDATE t SET id=3 WHERE id=1",
            "SELECT * FROM t;"
        ];
        for query in queries {
            let _ = session.execute(query)?;
        }
        session.execute("COMMIT")?;

        Ok(())
    }
}