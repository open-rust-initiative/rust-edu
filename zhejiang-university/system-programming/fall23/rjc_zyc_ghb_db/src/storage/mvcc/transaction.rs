use std::{sync::{Arc, Mutex, MutexGuard}, collections::HashSet, ops::{RangeBounds, Bound}};

use serde_derive::{Serialize, Deserialize};

use crate::{storage::{engine::Engine, bincode}, error::{Result, Error}};

use super::{key::{Version, Key, KeyPrefix}, iterator::Scan};

pub struct Transaction<E: Engine> {
    pub engine: Arc<Mutex<E>>,
    pub st: TransactionState,
}

/// 事务的状态
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TransactionState {
    pub version: Version,
    pub read_only: bool,
    pub active: HashSet<Version>,
}

impl TransactionState {
    pub fn is_visible(&self, version: Version) -> bool {
        if self.active.get(&version).is_some() {
            // 事物还没有提交，所以不可见
            false
        } else if self.read_only {
            version < self.version
        } else {
            version <= self.version
        }
    }
}

impl<E: Engine> Transaction<E> {
    pub fn begin(engine: Arc<Mutex<E>>) -> Result<Self> {
        let mut session = engine.lock()?;

        // 分配一个 version
        let version = match session.get(&Key::NextVersion.encode()?)? {
            Some(version) => bincode::deserialize(&version)?,
            None => 1,
        };
        session.set(&Key::NextVersion.encode()?, bincode::serialize(&(version + 1))?)?;

        // 将当前 version active 的 txn 持久化保存
        let active = Self::scan_active(&mut session)?;
        if !active.is_empty() {
            session.set(&Key::TxnActiveSnapshot(version).encode()?, bincode::serialize(&active)?)?;
        }
        // 将当前 version 标记为 active
        session.set(&Key::TxnActive(version).encode()?, vec![])?;
        drop(session);

        Ok(Self { engine, st: TransactionState { version, read_only: false, active } })
    }

    pub fn begin_read_only(engine: Arc<Mutex<E>>, as_of: Option<Version>) -> Result<Self> {
        let mut session = engine.lock()?;

        let mut version = match session.get(&Key::NextVersion.encode()?)? {
            Some(version) => bincode::deserialize(&version)?,
            None => 1,
        };

        let mut active = HashSet::new();
        if let Some(as_of) = as_of {
            if as_of >= version {
                return Err(Error::Value(format!("Version {} does not exist", as_of)));
            }
            version = as_of;
            if let Some(versions) = session.get(&Key::TxnActiveSnapshot(version).encode()?)? {
                active = bincode::deserialize(&versions)?;
            }
        } else {
            // 保持和最后一个 read-write version 一致
            active = Self::scan_active(&mut session)?;
        }

        drop(session);

        Ok(Self { engine, st: TransactionState { version, read_only: true, active } })
    }

    fn scan_active(session: &mut MutexGuard<E>) -> Result<HashSet<Version>> {
        let mut active = HashSet::new();
        let mut scan = session.scan_prefix(&KeyPrefix::TxnActive.encode()?);

        while let Some((key, _)) = scan.next().transpose()? {
            match Key::decode(&key)? {
                Key::TxnActive(version) => active.insert(version),
                _ => return Err(Error::Internal(format!("Expected TxnActive key, got {:?}", key))),
            };
        }

        Ok(active)
    }

    /// Resumes a transaction from the given state.
    pub fn resume(engine: Arc<Mutex<E>>, s: TransactionState) -> Result<Self> {
        // For read-write transactions, verify that the transaction is still
        // active before making further writes.
        if !s.read_only && engine.lock()?.get(&Key::TxnActive(s.version).encode()?)?.is_none() {
            return Err(Error::Internal(format!("No active transaction at version {}", s.version)));
        }
        Ok(Self { engine, st: s })
    }

    pub fn version(&self) -> Version {
        self.st.version
    }

    pub fn read_only(&self) -> bool {
        self.st.read_only
    }

    pub fn state(&self) -> &TransactionState {
        &self.st
    }

    pub fn commit(&self) -> Result<()> {
        if self.st.read_only {
            return Ok(());
        }

        let mut session = self.engine.lock()?;

        let remove = session.scan_prefix(&KeyPrefix::TxnWrite(self.version()).encode()?)
            .map(|r| r.map(|(k, _)| k))
            .collect::<Result<Vec<_>>>()?;
        for key in remove {
            session.delete(&key)?
        }

        session.delete(&Key::TxnActive(self.version()).encode()?)
    }

    pub fn rollback(&self) -> Result<()> {
        if self.st.read_only {
            return Ok(());
        }

        let mut session = self.engine.lock()?;

        let remove = session.scan_prefix(&KeyPrefix::TxnWrite(self.version()).encode()?)
            .map(|r| r.map(|(k, _)| k))
            .collect::<Result<Vec<_>>>()?;

        for key in remove {
            match Key::decode(&key)? {
                Key::TxnWrite(_, key) => {
                    let version = Key::Version(key.into(), self.version());
                    session.delete(&version.encode()?)?;
                },
                key => return Err(Error::Internal(format!("Expected TxnWrite, got {:?}", key))),
            }
        }

        session.delete(&Key::TxnActive(self.st.version).encode()?)
    }

    pub fn delete(&self, key: &[u8]) -> Result<()> {
        self.write_version(key, None)
    }

    pub fn set(&self, key: &[u8], value: Vec<u8>) -> Result<()> {
        self.write_version(key, Some(value))
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let mut session = self.engine.lock()?;
        let from = Key::Version(key.into(), 0).encode()?;
        let to = Key::Version(key.into(), self.st.version).encode()?;
        let mut scan = session.scan(from..=to).rev();
        while let Some((key, value)) = scan.next().transpose()? {
            match Key::decode(&key)? {
                Key::Version(_, version) => {
                    if self.st.is_visible(version) {
                        return bincode::deserialize(&value);
                    }
                }
                key => return Err(Error::Internal(format!("Expected Key::Version got {:?}", key))),
            };
        }
        Ok(None)
    }

    fn write_version(&self, key: &[u8], value: Option<Vec<u8>>) -> Result<()> {
        if self.st.read_only {
            return Err(Error::ReadOnly);
        }

        let mut session = self.engine.lock()?;

        // [from, to] 的事务对 key 的修改对当前来说都看不到
        let from = Key::Version(
            key.into(),
            self.st.active.iter().min().copied().unwrap_or(self.st.version + 1)
        ).encode()?;
        let to = Key::Version(key.into(), u64::MAX).encode()?;
        
        if let Some((key, _)) = session.scan(from..to).last().transpose()? {
            match Key::decode(&key)? {
                Key::Version(_, version) => {
                    if !self.st.is_visible(version) {
                        // 当前 key 存在对于 version 事务来说不可见的一些修改，直接返回错误
                        return Err(Error::Serialization);
                    }
                }
                key => return Err(Error::Internal(format!("Expected Key::Version got {:?}", key))),
            }
        }

        // 记录自己事务中发生了一些 write 事件
        session.set(&Key::TxnWrite(self.st.version, key.into()).encode()?, vec![])?;
        
        // 记录 key 对应的写事件
        session.set(&Key::Version(key.into(), self.st.version).encode()?, bincode::serialize(&value)?)
    }

    pub fn scan<R: RangeBounds<Vec<u8>>>(&self, range: R) -> Result<Scan<E>> {
        let start = match range.start_bound() {
            Bound::Excluded(k) => Bound::Excluded(Key::Version(k.into(), u64::MAX).encode()?),
            Bound::Included(k) => Bound::Included(Key::Version(k.into(), 0).encode()?),
            Bound::Unbounded => Bound::Included(Key::Version(vec![].into(), 0).encode()?),
        };
        let end = match range.end_bound() {
            Bound::Excluded(k) => Bound::Excluded(Key::Version(k.into(), 0).encode()?),
            Bound::Included(k) => Bound::Included(Key::Version(k.into(), u64::MAX).encode()?),
            Bound::Unbounded => Bound::Excluded(KeyPrefix::Unversioned.encode()?),
        };
        Ok(Scan::from_range(self.engine.lock()?, self.state(), start, end))
    }

    /// Scans keys under a given prefix.
    pub fn scan_prefix(&self, prefix: &[u8]) -> Result<Scan<E>> {
        // Normally, KeyPrefix::Version will only match all versions of the
        // exact given key. We want all keys maching the prefix, so we chop off
        // the KeyCode byte slice terminator 0x0000 at the end.
        let mut prefix = KeyPrefix::Version(prefix.into()).encode()?;
        prefix.truncate(prefix.len() - 2);
        Ok(Scan::from_prefix(self.engine.lock()?, self.state(), prefix))
    }
}