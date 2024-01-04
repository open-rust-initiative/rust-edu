use std::{path::PathBuf, fs::{File, self}, io::{BufReader, Seek, SeekFrom, Read, BufWriter, Write}};

use crate::error::Result;

use super::KeyDir;

pub(crate) struct Log {
    pub(crate) path: PathBuf,
    pub(crate) file: File,
}

impl Log {
    pub(crate) fn new(path: PathBuf) -> Result<Log> {
        if let Some(path) = path.parent() {
            fs::create_dir_all(path)?;
        }
        let file = std::fs::OpenOptions::new().read(true).write(true).create(true).open(&path)?;
        Ok(Log { path, file })
    }

    pub(crate) fn build_keydir(&mut self) -> Result<KeyDir> {
        let mut len_buf = [0u8; 4];
        let mut keydir = KeyDir::new();
        let file_len = self.file.metadata()?.len();
        let mut r = BufReader::new(&mut self.file);
        let mut pos = r.seek(SeekFrom::Start(0))?;

        while pos < file_len {
            let result = || -> std::result::Result<(Vec<u8>, u64, Option<u32>), std::io::Error> {
                r.read_exact(&mut len_buf)?;
                let key_len = u32::from_be_bytes(len_buf);
                r.read_exact(&mut len_buf)?;
                let value_len_or_tombstone = match i32::from_be_bytes(len_buf) {
                    l if l >= 0 => Some(l as u32),
                    _ => None,
                };

                // | key len 4bytes | value len 4bytes | key | value |
                //                                           ^
                //                                        value_pos
                let value_pos = pos + 4 + 4 + key_len as u64;

                let mut key = vec![0; key_len as usize];
                r.read_exact(&mut key)?;

                if let Some(value_len) = value_len_or_tombstone {
                    if value_pos + value_len as u64 > file_len {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "value extends beyond end of file",
                        ));
                    }
                    r.seek_relative(value_len as i64)?; // avoids discarding buffer
                }

                Ok((key, value_pos, value_len_or_tombstone))
            }();

            match result {
                Ok((key, value_pos, Some(value_len))) => {
                    keydir.insert(key, (value_pos, value_len));
                    pos = value_pos + value_len as u64;
                }
                Ok((key, value_pos, None)) => {
                    keydir.remove(&key);
                    pos = value_pos;
                }
                Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    log::error!("Found incomplete entry at offset {}, truncating file", pos);
                    self.file.set_len(pos)?;
                    break;
                }
                Err(err) => return Err(err.into()),
            }
        }

        Ok(keydir)
    }

    pub(crate) fn read_value(&mut self, value_pos: u64, value_len: u32) -> Result<Vec<u8>> {
        let mut value_buf = vec![0; value_len as usize];
        self.file.seek(SeekFrom::Start(value_pos))?;
        self.file.read_exact(&mut value_buf)?;

        Ok(value_buf)
    }

    /// 将 key value 写入 log 文件中，返回记录起始位置 pos 和记录的长度 len
    pub(crate) fn write_entry(&mut self, key: &[u8], value: Option<&[u8]>) -> Result<(u64, u32)> {
        let key_len = key.len() as u32;
        let value_len = value.map_or(0, |v| v.len() as u32);
        let value_len_or_tombstone = value.map_or(-1, |v| v.len() as i32);
        let len = 4 + 4 + key_len + value_len;


        let pos = self.file.seek(SeekFrom::End(0))?;
        let mut writer = BufWriter::with_capacity(len as usize, &mut self.file);

        writer.write_all(&key_len.to_be_bytes())?;
        writer.write_all(&value_len_or_tombstone.to_be_bytes())?;
        writer.write_all(key)?;

        if let Some(value) = value {
            writer.write_all(value)?;
        }
        writer.flush()?;
        Ok((pos, len))
    }

    pub(crate) fn total_size(&self) -> Result<u64> {
        Ok(self.file.metadata()?.len())
    }

    pub(crate) fn flush(&self) -> Result<()> {
        self.file.sync_all()?;
        Ok(())
    }
}