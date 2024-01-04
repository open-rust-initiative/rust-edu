use serde_derive::{Serialize, Deserialize};

use crate::error::Result;

pub mod bitcask;

mod log;

mod iterator;

type KeyDir = std::collections::BTreeMap<Vec<u8>, (u64, u32)>;

/// Engine status.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Status {
    /// engine name
    pub name: String,
    /// live key 数量
    pub keys: u64,
    /// live key value byte 的大小
    pub size: u64,
    /// 总体大小
    pub total_disk_size: u64,
    /// live data 大小
    pub live_disk_size: u64,
    /// garbage data 大小
    pub garbage_disk_size: u64,
}

pub trait Engine: std::fmt::Display + Send + Sync {
    type ScanIterator<'a>: DoubleEndedIterator<Item = Result<(Vec<u8>, Vec<u8>)>> + 'a
    where
        Self: 'a;
    
    fn delete(&mut self, key: &[u8]) -> Result<()>;

    fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    fn set(&mut self, key: &[u8], value: Vec<u8>) -> Result<()>;

    fn flush(&mut self) -> Result<()>;

    fn scan<R: std::ops::RangeBounds<Vec<u8>>>(&mut self, range: R) -> Self::ScanIterator<'_>;

    fn status(&mut self) -> Result<Status>;

    fn scan_prefix(&mut self, prefix: &[u8]) -> Self::ScanIterator<'_> {
        let start = std::ops::Bound::Included(prefix.to_vec());
        let end = match prefix.iter().rposition(|b| *b != 0xff) {
            Some(i) => std::ops::Bound::Excluded(
                prefix.iter().take(i).copied().chain(std::iter::once(prefix[i] + 1)).collect(),
            ),
            None => std::ops::Bound::Unbounded,
        };
        self.scan((start, end))
    }
}