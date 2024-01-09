use std::{sync::MutexGuard, ops::Bound};

use crate::{storage::{engine::Engine, bincode}, error::{Result, Error}};

use super::{transaction::TransactionState, key::{Version, Key}};

pub struct Scan<'a, E: Engine + 'a> {
    /// Access to the locked engine.
    engine: MutexGuard<'a, E>,
    /// The transaction state.
    txn: &'a TransactionState,
    /// The scan type and parameter.
    param: ScanType,
}

enum ScanType {
    Range((Bound<Vec<u8>>, Bound<Vec<u8>>)),
    Prefix(Vec<u8>),
}

impl<'a, E: Engine + 'a> Scan<'a, E> {
    /// Runs a normal range scan.
    pub fn from_range(
        engine: MutexGuard<'a, E>,
        txn: &'a TransactionState,
        start: Bound<Vec<u8>>,
        end: Bound<Vec<u8>>,
    ) -> Self {
        Self { engine, txn, param: ScanType::Range((start, end)) }
    }

    /// Runs a prefix scan.
    pub fn from_prefix(engine: MutexGuard<'a, E>, txn: &'a TransactionState, prefix: Vec<u8>) -> Self {
        Self { engine, txn, param: ScanType::Prefix(prefix) }
    }

    /// Returns an iterator over the result.
    pub fn iter(&mut self) -> ScanIterator<'_, E> {
        let inner = match &self.param {
            ScanType::Range(range) => self.engine.scan(range.clone()),
            ScanType::Prefix(prefix) => self.engine.scan_prefix(prefix),
        };
        ScanIterator::new(self.txn, inner)
    }

    /// Collects the result to a vector.
    pub fn to_vec(&mut self) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.iter().collect()
    }
}

/// An iterator over the latest live and visible key/value pairs at the txn
/// version.
pub struct ScanIterator<'a, E: Engine + 'a> {
    /// Decodes and filters visible MVCC versions from the inner engine iterator.
    inner: std::iter::Peekable<VersionIterator<'a, E>>,
    /// The previous key emitted by try_next_back(). Note that try_next() does
    /// not affect reverse positioning: double-ended iterators consume from each
    /// end independently.
    last_back: Option<Vec<u8>>,
}

impl<'a, E: Engine + 'a> ScanIterator<'a, E> {
    /// Creates a new scan iterator.
    fn new(txn: &'a TransactionState, inner: E::ScanIterator<'a>) -> Self {
        Self { inner: VersionIterator::new(txn, inner).peekable(), last_back: None }
    }

    /// Fallible next(), emitting the next item, or None if exhausted.
    fn try_next(&mut self) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
        while let Some((key, _version, value)) = self.inner.next().transpose()? {
            // If the next key equals this one, we're not at the latest version.
            match self.inner.peek() {
                Some(Ok((next, _, _))) if next == &key => continue,
                Some(Err(err)) => return Err(err.clone()),
                Some(Ok(_)) | None => {}
            }
            // If the key is live (not a tombstone), emit it.
            if let Some(value) = bincode::deserialize(&value)? {
                return Ok(Some((key, value)));
            }
        }
        Ok(None)
    }

    /// Fallible next_back(), emitting the next item from the back, or None if
    /// exhausted.
    fn try_next_back(&mut self) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
        while let Some((key, _version, value)) = self.inner.next_back().transpose()? {
            // If this key is the same as the last emitted key from the back,
            // this must be an older version, so skip it.
            if let Some(last) = &self.last_back {
                if last == &key {
                    continue;
                }
            }
            self.last_back = Some(key.clone());

            // If the key is live (not a tombstone), emit it.
            if let Some(value) = bincode::deserialize(&value)? {
                return Ok(Some((key, value)));
            }
        }
        Ok(None)
    }
}

impl<'a, E: Engine> Iterator for ScanIterator<'a, E> {
    type Item = Result<(Vec<u8>, Vec<u8>)>;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}

impl<'a, E: Engine> DoubleEndedIterator for ScanIterator<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.try_next_back().transpose()
    }
}

/// An iterator that decodes raw engine key/value pairs into MVCC key/value
/// versions, and skips invisible versions. Helper for ScanIterator.
struct VersionIterator<'a, E: Engine + 'a> {
    /// The transaction the scan is running in.
    txn: &'a TransactionState,
    /// The inner engine scan iterator.
    inner: E::ScanIterator<'a>,
}

#[allow(clippy::type_complexity)]
impl<'a, E: Engine + 'a> VersionIterator<'a, E> {
    /// Creates a new MVCC version iterator for the given engine iterator.
    fn new(txn: &'a TransactionState, inner: E::ScanIterator<'a>) -> Self {
        Self { txn, inner }
    }

    /// Decodes a raw engine key into an MVCC key and version, returning None if
    /// the version is not visible.
    fn decode_visible(&self, key: &[u8]) -> Result<Option<(Vec<u8>, Version)>> {
        let (key, version) = match Key::decode(key)? {
            Key::Version(key, version) => (key.into_owned(), version),
            key => return Err(Error::Internal(format!("Expected Key::Version got {:?}", key))),
        };
        if self.txn.is_visible(version) {
            Ok(Some((key, version)))
        } else {
            Ok(None)
        }
    }

    // Fallible next(), emitting the next item, or None if exhausted.
    fn try_next(&mut self) -> Result<Option<(Vec<u8>, Version, Vec<u8>)>> {
        while let Some((key, value)) = self.inner.next().transpose()? {
            if let Some((key, version)) = self.decode_visible(&key)? {
                return Ok(Some((key, version, value)));
            }
        }
        Ok(None)
    }

    // Fallible next_back(), emitting the previous item, or None if exhausted.
    fn try_next_back(&mut self) -> Result<Option<(Vec<u8>, Version, Vec<u8>)>> {
        while let Some((key, value)) = self.inner.next_back().transpose()? {
            if let Some((key, version)) = self.decode_visible(&key)? {
                return Ok(Some((key, version, value)));
            }
        }
        Ok(None)
    }
}

impl<'a, E: Engine> Iterator for VersionIterator<'a, E> {
    type Item = Result<(Vec<u8>, Version, Vec<u8>)>;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}

impl<'a, E: Engine> DoubleEndedIterator for VersionIterator<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.try_next_back().transpose()
    }
}
