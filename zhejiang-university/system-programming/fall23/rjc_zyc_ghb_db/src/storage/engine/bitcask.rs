use std::{path::PathBuf, fs};

use crate::error::Result;

use super::{log::Log, KeyDir, iterator::ScanIterator, Status, Engine};

pub struct Bitcask {
    log: Log,
    keydir: KeyDir,
}

impl Bitcask {
    pub fn new(path: PathBuf) -> Result<Bitcask> {
        let mut log = Log::new(path)?;
        let keydir = log.build_keydir()?;
        Ok(Bitcask { log, keydir })
    }

    pub fn new_compact(path: PathBuf, garbage_ratio_threshold: f64) -> Result<Bitcask> {
        let mut bitcask = Bitcask::new(path)?;
        
        let status = bitcask.status()?;

        let garbage_ratio = status.garbage_disk_size as f64 / status.total_disk_size as f64;
        if status.garbage_disk_size > 0 && garbage_ratio >= garbage_ratio_threshold {
            log::info!(
                "Compacting {} to remove {:.3}MB garbage ({:.0}% of {:.3}MB)",
                bitcask.log.path.display(),
                status.garbage_disk_size / 1024 / 1024,
                garbage_ratio * 100.0,
                status.total_disk_size / 1024 / 1024
            );
            bitcask.compact()?;
            log::info!(
                "Compacted {} to size {:.3}MB",
                bitcask.log.path.display(),
                (status.total_disk_size - status.garbage_disk_size) / 1024 / 1024
            );
        }

        Ok(bitcask)
    }
}

impl Engine for Bitcask {
    type ScanIterator<'a> = ScanIterator<'a>;

    fn delete(&mut self, key: &[u8]) -> Result<()> {
        let _ = self.log.write_entry(key, None)?;
        self.keydir.remove(key);
        Ok(())
    }

    fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        if let Some((value_pos, value_len)) = self.keydir.get(key) {
            Ok(Some(self.log.read_value(*value_pos, *value_len)?))
        } else {
            Ok(None)
        }
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) -> Result<()> {
        let (pos, len) = self.log.write_entry(key, Some(&value))?;
        let value_len = value.len() as u32;
        self.keydir.insert(key.to_vec(), (pos + len as u64 - value_len as u64, value_len));
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.log.flush()
    }

    fn scan<R: std::ops::RangeBounds<Vec<u8>>>(&mut self, range: R) -> Self::ScanIterator<'_> {
        ScanIterator::new(self.keydir.range(range), &mut self.log)
    }

    fn status(&mut self) -> Result<Status> {
        let name = self.to_string();
        let keys = self.keydir.len() as u64;

        let size = self.keydir
                            .iter()
                            .fold(0 as u64, |size, (key, (_, value_len))| size + key.len() as u64 + *value_len as u64);
        
        let total_disk_size = self.log.total_size()?;

        let live_disk_size = size + 8 * keys;

        let garbage_disk_size = total_disk_size - live_disk_size;

        Ok(Status {
            name,
            keys,
            size,
            total_disk_size,
            live_disk_size,
            garbage_disk_size,
        })        
    }
}

impl std::fmt::Display for Bitcask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bitcask")
    }
}

/// 与 Bitcask 压缩有关的函数
impl Bitcask {
    /// 将 bitcask 进行压缩
    pub fn compact(&mut self) -> Result<()> {
        let mut tmp_path = self.log.path.clone();
        let _ = tmp_path.set_extension("compact");

        let (mut new_log, new_keydir) = self.write_log(tmp_path)?;

        fs::rename(&new_log.path, &self.log.path)?;
        new_log.path = self.log.path.clone();
        self.log = new_log;
        self.keydir = new_keydir;

        Ok(())
    }

    /// 重新写入一份 log，并且构造对应的 keydir
    fn write_log(&mut self, path: PathBuf) -> Result<(Log, KeyDir)> {
        let mut new_log = Log::new(path)?;
        let mut new_keydir = KeyDir::new();

        for (key, (value_pos, value_len)) in self.keydir.iter() {
            let value = self.log.read_value(*value_pos, *value_len)?;
            let (new_pos, new_len) = new_log.write_entry(key, Some(&value))?;
            new_keydir.insert(key.to_vec(), (new_pos, new_len));
        }

        Ok((new_log, new_keydir))
    }
}

#[cfg(test)]
mod tests {
    use crate::{storage::engine::Engine, error::Result};

    use super::Bitcask;

    #[test]
    fn log() -> crate::Result<()> {
        let path = tempdir::TempDir::new("waterdb")?.path().join("waterdb");
        let mut s = Bitcask::new(path.clone())?;
        s.set(b"b", vec![0x01])?;
        s.set(b"b", vec![0x02])?;

        s.set(b"e", vec![0x05])?;
        s.delete(b"e")?;

        s.set(b"c", vec![0x00])?;
        s.delete(b"c")?;
        s.set(b"c", vec![0x03])?;

        s.set(b"", vec![])?;

        s.set(b"a", vec![0x01])?;

        s.delete(b"f")?;

        s.delete(b"d")?;
        s.set(b"d", vec![0x04])?;

        // Make sure the scan yields the expected results.
        assert_eq!(
            vec![
                (b"".to_vec(), vec![]),
                (b"a".to_vec(), vec![0x01]),
                (b"b".to_vec(), vec![0x02]),
                (b"c".to_vec(), vec![0x03]),
                (b"d".to_vec(), vec![0x04]),
            ],
            s.scan(..).collect::<Result<Vec<_>>>()?,
        );

        Ok(())
    }
}