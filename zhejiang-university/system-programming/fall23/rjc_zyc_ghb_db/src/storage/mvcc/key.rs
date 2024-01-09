use std::borrow::Cow;

use serde_derive::{Deserialize, Serialize};

use crate::{error::Result, storage::keycode};

/// 表示一个事物的逻辑时间戳
pub type Version = u64;

/// 
#[derive(Debug, Deserialize, Serialize)]
pub enum Key<'a> {
    /// 下一个可用的 Version
    NextVersion,
    /// 目前 active 的事务
    TxnActive(Version),
    /// version 事务来说 active 的事务
    TxnActiveSnapshot(Version),
    /// 对事务而言，记录事务操作了哪些 key
    /// 方便之后回滚操作
    TxnWrite(
        Version,
        #[serde(with = "serde_bytes")]
        #[serde(borrow)]
        Cow<'a, [u8]>,
    ),
    /// 对 key 而言，记录哪些事务操作了这个 key
    Version(
        #[serde(with = "serde_bytes")]
        #[serde(borrow)]
        Cow<'a, [u8]>,
        Version,
    ),
    /// Unversioned non-transactional key/value pairs. These exist separately
    /// from versioned keys, i.e. the unversioned key "foo" is entirely
    /// independent of the versioned key "foo@7". These are mostly used
    /// for metadata.
    Unversioned(
        #[serde(with = "serde_bytes")]
        #[serde(borrow)]
        Cow<'a, [u8]>,
    ),
}

impl<'a> Key<'a> {
    pub fn decode(bytes: &'a [u8]) -> Result<Self> {
        keycode::deserialize(bytes)
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        keycode::serialize(&self)
    }
}

/// MVCC key prefixes, for prefix scans. These must match the keys above,
/// including the enum variant index.
#[derive(Debug, Deserialize, Serialize)]
pub enum KeyPrefix<'a> {
    NextVersion,
    TxnActive,
    TxnActiveSnapshot,
    TxnWrite(Version),
    Version(
        #[serde(with = "serde_bytes")]
        #[serde(borrow)]
        Cow<'a, [u8]>,
    ),
    Unversioned,
}

impl<'a> KeyPrefix<'a> {
    pub fn encode(&self) -> Result<Vec<u8>> {
        keycode::serialize(&self)
    }
}