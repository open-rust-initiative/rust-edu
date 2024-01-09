/// 将实现 MVCC，MVCC 广泛用于保证 ACID 以及并发控制。
/// 使得多个事务可以同时隔离的并发访问同一个数据集，并且处理冲突，
/// 当事务 commit 的时候，实现原子性写入
use std::sync::{Arc, Mutex};

use serde_derive::{Serialize, Deserialize};

use crate::{storage::engine::Engine, error::Result};

use super::{transaction::{Transaction, TransactionState}, key::{Version, Key, KeyPrefix}};

pub struct MVCC<E: Engine> {
    engine: Arc<Mutex<E>>,
}

/// MVCC engine 状态
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Status {
    /// MVCC 事务总数
    pub versions: u64,
    /// 目前存活事务的数量
    pub active_txns: u64,
    /// engine 的状态
    pub storage: crate::storage::engine::Status,
}

impl<E: Engine> Clone for MVCC<E> {
    fn clone(&self) -> Self {
        MVCC { engine: self.engine.clone() }
    }
}

impl<E: Engine> MVCC<E> {
    pub fn new(engine: E) -> Self {
        Self { engine: Arc::new(Mutex::new(engine)) }
    }

    pub fn begin(&self) -> Result<Transaction<E>> {
        Transaction::begin(self.engine.clone())
    }

    pub fn begin_read_only(&self) -> Result<Transaction<E>> {
        Transaction::begin_read_only(self.engine.clone(), None)
    }

    pub fn begin_as_of(&self, version: Version) -> Result<Transaction<E>> {
        Transaction::begin_read_only(self.engine.clone(), Some(version))
    }

    pub fn resume(&self, state: TransactionState) -> Result<Transaction<E>> {
        Transaction::resume(self.engine.clone(), state)
    }

    pub fn get_unversioned(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.engine.lock()?.get(&Key::Unversioned(key.into()).encode()?)
    }

    pub fn set_unversioned(&self, key: &[u8], value: Vec<u8>) -> Result<()> {
        self.engine.lock()?.set(&Key::Unversioned(key.into()).encode()?, value)
    }

    pub fn status(&self) -> Result<Status> {
        let mut engine = self.engine.lock()?;
        let versions = match engine.get(&Key::NextVersion.encode()?)? {
            Some(ref v) => bincode::deserialize::<u64>(v)? - 1,
            None => 0,
        };
        let active_txns = engine.scan_prefix(&KeyPrefix::TxnActive.encode()?).count() as u64;
        Ok(Status { versions, active_txns, storage: engine.status()? })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, vec};

    use crate::{storage::{engine::bitcask::Bitcask, mvcc::transaction::TransactionState}, error::{Result, Error}};

    use super::MVCC;

    macro_rules! assert_scan {
        ( $scan:expr => { $( $key:expr => $value:expr),* $(,)? } ) => {
            let result = $scan.to_vec()?;
            let expect = vec![
                $( ($key.to_vec(), $value.to_vec()), )*
            ];
            assert_eq!(result, expect);
        };
    }

    #[test]
    fn begin() -> Result<()> {
        let path = tempdir::TempDir::new("waterdb")?.path().join("waterdb");
        let engine = Bitcask::new(path)?;
        let mvcc = MVCC::new(engine);

        let t1 = mvcc.begin()?;
        assert_eq!(
            *t1.state(),
            TransactionState { version: 1, read_only: false, active: HashSet::new() }
        );

        let t2 = mvcc.begin()?;
        assert_eq!(
            *t2.state(),
            TransactionState { version: 2, read_only: false, active: HashSet::from([1]) }
        );

        let t3 = mvcc.begin()?;
        assert_eq!(
            *t3.state(),
            TransactionState { version: 3, read_only: false, active: HashSet::from([1, 2]) }
        );

        t2.commit()?;

        let t4 = mvcc.begin()?;
        assert_eq!(
            *t4.state(),
            TransactionState { version: 4, read_only: false, active: HashSet::from([1, 3]) }
        );

        Ok(())
    }

    #[test]
    fn read_only() -> Result<()> {
        let path = tempdir::TempDir::new("waterdb")?.path().join("waterdb");
        let engine = Bitcask::new(path)?;
        let mvcc = MVCC::new(engine);

        let t1 = mvcc.begin_read_only()?;
        assert_eq!(
            *t1.state(),
            TransactionState { version: 1, read_only: true, active: HashSet::new() }
        );
        assert_eq!(t1.set(b"foo", vec![1]), Err(Error::ReadOnly));
        assert_eq!(t1.delete(b"foo"), Err(Error::ReadOnly));

        let t2 = mvcc.begin()?;
        assert_eq!(
            *t2.state(),
            TransactionState { version: 1, read_only: false, active: HashSet::new() }
        );

        let t3 = mvcc.begin_read_only()?;
        assert_eq!(
            *t3.state(),
            TransactionState { version: 2, read_only: true, active: HashSet::from([1]) }
        );

        Ok(())
    }

    #[test]
    fn as_of() -> Result<()> {
        let path = tempdir::TempDir::new("waterdb")?.path().join("waterdb");
        let engine = Bitcask::new(path)?;
        let mvcc = MVCC::new(engine);

        let t1 = mvcc.begin()?;
        t1.set(b"other", vec![1])?;

        let t2 = mvcc.begin()?;
        t2.set(b"key", vec![2])?;
        t2.commit()?;

        let t3 = mvcc.begin()?;
        t3.set(b"key", vec![3])?;

        let t4 = mvcc.begin_as_of(3)?;
        assert_eq!(
            *t4.state(),
            TransactionState { version: 3, read_only: true, active: HashSet::from([1]) }
        );
        
        assert_scan!(t4.scan(..)? => {b"key" => [2]});

        assert_eq!(t4.set(b"foo", vec![1]), Err(Error::ReadOnly));
        assert_eq!(t4.delete(b"foo"), Err(Error::ReadOnly));

        t1.commit()?;
        t3.commit()?;


        assert_scan!(t4.scan(..)? => {b"key" => [2]});

        let t5 = mvcc.begin_as_of(3)?;
        assert_scan!(t5.scan(..)? => {b"key" => [2]});

        t4.rollback()?;
        t5.commit()?;

        let t6 = mvcc.begin()?;
        t6.set(b"key", vec![4])?;
        t6.commit()?;

        let t7 = mvcc.begin_as_of(4)?;
        assert_eq!(
            *t7.state(),
            TransactionState { version: 4, read_only: true, active: HashSet::new() }
        );
        assert_scan!(t7.scan(..)? => {b"key" => [3], b"other" => [1]});

        assert_eq!(
            mvcc.begin_as_of(5).err(),
            Some(Error::Value("Version 5 does not exist".into()))
        );
        assert_eq!(
            mvcc.begin_as_of(9).err(),
            Some(Error::Value("Version 9 does not exist".into()))
        );

        Ok(())
    }

    #[test]
    fn delete_conflict() -> Result<()> {
        let path = tempdir::TempDir::new("waterdb")?.path().join("waterdb");
        let engine = Bitcask::new(path)?;
        let mvcc = MVCC::new(engine);
        
        let t1 = mvcc.begin()?;
        let t2 = mvcc.begin()?;
        let t3 = mvcc.begin()?;
        let t4 = mvcc.begin()?;

        t1.set(b"a", vec![1])?;
        t3.set(b"c", vec![3])?;
        t4.set(b"d", vec![4])?;
        t4.commit()?;

        assert_eq!(t2.delete(b"a"), Err(Error::Serialization));
        assert_eq!(t2.delete(b"c"), Err(Error::Serialization));
        assert_eq!(t2.delete(b"d"), Err(Error::Serialization));

        Ok(())
    }

    #[test]
    fn get() -> Result<()> {
        let path = tempdir::TempDir::new("waterdb")?.path().join("waterdb");
        let engine = Bitcask::new(path)?;
        let mvcc = MVCC::new(engine);
        
        let t = mvcc.begin()?;
        t.set(b"key", vec![1])?;
        t.set(b"updated", vec![1])?;
        t.set(b"updated",vec![2])?;
        t.set(b"deleted", vec![1])?;
        t.delete(b"deleted")?;
        t.commit()?;

        let t1 = mvcc.begin()?;
        assert_eq!(t1.get(b"key")?, Some(vec![1]));
        assert_eq!(t1.get(b"updated")?, Some(vec![2]));
        assert_eq!(t1.get(b"deleted")?, None);
        Ok(())
    }

    #[test]
    fn get_isolation() -> Result<()> {
        let path = tempdir::TempDir::new("waterdb")?.path().join("waterdb");
        let engine = Bitcask::new(path)?;
        let mvcc = MVCC::new(engine);

        let t1 = mvcc.begin()?;
        t1.set(b"a", vec![1])?;
        t1.set(b"b", vec![1])?;
        t1.set(b"d", vec![1])?;
        t1.set(b"e", vec![1])?;
        t1.commit()?;

        let t2 = mvcc.begin()?;
        t2.set(b"a", vec![2])?;
        t2.delete(b"b")?;
        t2.set(b"c", vec![2])?;

        let t3 = mvcc.begin_read_only()?;

        let t4 = mvcc.begin()?;
        t4.set(b"d", vec![3])?;
        t4.delete(b"e")?;
        t4.set(b"f", vec![3])?;
        t4.commit()?;

        assert_eq!(t3.get(b"a")?, Some(vec![1])); // uncommitted update
        assert_eq!(t3.get(b"b")?, Some(vec![1])); // uncommitted delete
        assert_eq!(t3.get(b"c")?, None); // uncommitted write
        assert_eq!(t3.get(b"d")?, Some(vec![1])); // future update
        assert_eq!(t3.get(b"e")?, Some(vec![1])); // future delete
        assert_eq!(t3.get(b"f")?, None); // future write

        Ok(())
    }

    #[test]
    fn set_conflict() -> Result<()> {
        let path = tempdir::TempDir::new("waterdb")?.path().join("waterdb");
        let engine = Bitcask::new(path)?;
        let mvcc = MVCC::new(engine);

        let t1 = mvcc.begin()?;
        let t2 = mvcc.begin()?;
        let t3 = mvcc.begin()?;
        let t4 = mvcc.begin()?;

        t1.set(b"a", vec![1])?;
        t3.set(b"c", vec![3])?;
        t4.set(b"d", vec![4])?;
        t4.commit()?;

        assert_eq!(t2.set(b"a", vec![2]), Err(Error::Serialization)); // past uncommitted
        assert_eq!(t2.set(b"c", vec![2]), Err(Error::Serialization)); // future uncommitted
        assert_eq!(t2.set(b"d", vec![2]), Err(Error::Serialization)); // future committed

        Ok(())
    }

    #[test]
    fn rollback() -> Result<()> {
        let path = tempdir::TempDir::new("waterdb")?.path().join("waterdb");
        let engine = Bitcask::new(path)?;
        let mvcc = MVCC::new(engine);

        let init = mvcc.begin()?;
        init.set(b"a", vec![0])?;
        init.set(b"b", vec![0])?;
        init.set(b"c", vec![0])?;
        init.set(b"d", vec![0])?;
        init.commit()?;

        let t1 = mvcc.begin()?;
        let t2 = mvcc.begin()?;
        let t3 = mvcc.begin()?;

        t1.set(b"a", vec![1])?;
        t2.set(b"b", vec![2])?;
        t2.delete(b"c")?;
        t3.set(b"d", vec![3])?;

        assert_eq!(t1.set(b"b", vec![1]), Err(Error::Serialization));
        assert_eq!(t3.set(b"c", vec![3]), Err(Error::Serialization));

        t2.rollback()?;

        let t4 = mvcc.begin_read_only()?;
        assert_scan!(t4.scan(..)? => {
            b"a" => [0],
            b"b" => [0],
            b"c" => [0],
            b"d" => [0],
        });

        t1.set(b"b", vec![1])?;
        t3.set(b"c", vec![3])?;
        t1.commit()?;
        t3.commit()?;

        let t5 = mvcc.begin_read_only()?;
        assert_scan!(t5.scan(..)? => {
            b"a" => [1],
            b"b" => [1],
            b"c" => [3],
            b"d" => [3],
        });

        Ok(())
    }
}