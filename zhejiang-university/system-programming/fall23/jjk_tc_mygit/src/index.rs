use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::cmp;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::convert::TryInto;
use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind, Read, Write};
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::str;

use crate::lockfile::Lockfile;
use crate::util::*;

const MAX_PATH_SIZE: u16 = 0xfff;
const CHECKSUM_SIZE: u64 = 20;

const HEADER_SIZE: usize = 12; // bytes
const MIN_ENTRY_SIZE: usize = 36;

#[derive(Debug, Clone)]
pub struct Entry {
    ctime: u64,
    mtime: u64,
    size: u64,
    flags: u16,
    pub oid: String,
    pub path: String,
}

impl Entry {
    fn is_executable(mode: u32) -> bool {
        (mode >> 6) & 0b1 == 1
    }

    fn mode(mode: u32) -> u32 {
        if Entry::is_executable(mode) {
            0o100755u32
        } else {
            0o100644u32
        }
    }
    fn new(pathname: &str, oid: &str, metadata: &fs::Metadata) -> Entry {
        let path = pathname.to_string();
        Entry {
            ctime: metadata.creation_time(),
            mtime: metadata.last_write_time(),
            size: metadata.file_size(),
            oid: oid.to_string(),
            flags: cmp::min(path.len() as u16, MAX_PATH_SIZE),
            path,
        }
    }

    fn parse(bytes: &[u8]) -> Result<Entry, std::io::Error> {
        let mut metadata_ints: Vec<u32> = vec![];
        for i in 0..3 {
            metadata_ints.push(u32::from_be_bytes(
                bytes[i * 4..i * 4 + 4].try_into().unwrap(),
            ));
        }

        let oid = encode_hex(&bytes[12..32]);
        let flags = u16::from_be_bytes(bytes[32..34].try_into().unwrap());
        let path_bytes = bytes[34..].split(|b| b == &0u8).next().unwrap();
        let path = str::from_utf8(path_bytes).unwrap().to_string();

        Ok(Entry {
            ctime: u64::from(metadata_ints[0]),
            mtime: u64::from(metadata_ints[1]),
            size: u64::from(metadata_ints[2]),

            oid,
            flags,
            path,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // 10 32-bit integers
        bytes.extend_from_slice(&(self.ctime as u32).to_be_bytes());
        bytes.extend_from_slice(&(self.mtime as u32).to_be_bytes());
        bytes.extend_from_slice(&(self.size as u32).to_be_bytes());

        // 20 bytes (40-char hex-string)
        bytes.extend_from_slice(&decode_hex(&self.oid).expect("invalid oid"));

        // 16-bit
        bytes.extend_from_slice(&self.flags.to_be_bytes());

        bytes.extend_from_slice(self.path.as_bytes());
        bytes.push(0x0);

        // add padding
        while bytes.len() % 8 != 0 {
            bytes.push(0x0)
        }

        bytes
    }

    fn parent_dirs(&self) -> Vec<&str> {
        let path = Path::new(&self.path);
        let mut parent_dirs: Vec<_> = path
            .ancestors()
            .map(|d| d.to_str().expect("invalid filename"))
            .collect();
        parent_dirs.pop(); // drop root dir(always "")
        parent_dirs.reverse();
        parent_dirs.pop(); // drop filename

        parent_dirs
    }
}

pub struct Checksum<T>
where
    T: Read + Write,
{
    file: T,
    digest: Sha1,
}

impl<T> Checksum<T>
where
    T: Read + Write,
{
    fn new(file: T) -> Checksum<T> {
        Checksum {
            file,
            digest: Sha1::new(),
        }
    }

    fn read(&mut self, size: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut buf = vec![0; size];
        self.file.read_exact(&mut buf)?;
        self.digest.input(&buf);

        Ok(buf)
    }

    fn write(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        self.file.write_all(data)?;
        self.digest.input(data);

        Ok(())
    }

    fn write_checksum(&mut self) -> Result<(), std::io::Error> {
        self.file
            .write_all(&decode_hex(&self.digest.result_str()).unwrap())?;

        Ok(())
    }

    fn verify_checksum(&mut self) -> Result<(), std::io::Error> {
        let hash = self.digest.result_str();

        let mut buf = vec![0; CHECKSUM_SIZE as usize];
        self.file.read_exact(&mut buf)?;

        let sum = encode_hex(&buf);

        if sum != hash {
            return Err(io::Error::new(
                ErrorKind::Other,
                "Checksum does not match value stored on disk",
            ));
        }

        Ok(())
    }
}

pub struct Index {
    pathname: PathBuf,
    pub entries: BTreeMap<String, Entry>,
    parents: HashMap<String, HashSet<String>>,
    lockfile: Lockfile,
    hasher: Option<Sha1>,
    changed: bool,
}

impl Index {
    pub fn new(path: &Path) -> Index {
        Index {
            pathname: path.to_path_buf(),
            entries: BTreeMap::new(),
            parents: HashMap::new(),
            lockfile: Lockfile::new(path),
            hasher: None,
            changed: false,
        }
    }
    //4
    pub fn write_updates(&mut self) -> Result<(), std::io::Error> {
        if !self.changed {
            return self.lockfile.rollback();
        }

        let lock = &mut self.lockfile;
        let mut writer: Checksum<&Lockfile> = Checksum::new(lock);

        let mut header_bytes: Vec<u8> = vec![];
        header_bytes.extend_from_slice(b"DIRC");
        header_bytes.extend_from_slice(&2u32.to_be_bytes()); // version no.
        header_bytes.extend_from_slice(&(self.entries.len() as u32).to_be_bytes());
        writer.write(&header_bytes)?;
        for (_key, entry) in self.entries.iter() {
            writer.write(&entry.to_bytes())?;
        }
        writer.write_checksum()?;
        lock.commit()?;

        Ok(())
    }

    /// Remove any entries whose name matches the name of one of the
    /// new entry's parent directories
    pub fn discard_conflicts(&mut self, entry: &Entry) {
        for parent in entry.parent_dirs() {
            self.remove_entry(parent);
        }

        let to_remove = {
            let mut children = vec![];
            if let Some(children_set) = self.parents.get(&entry.path) {
                for child in children_set {
                    children.push(child.clone())
                }
            }
            children
        };

        for child in to_remove {
            self.remove_entry(&child);
        }
    }

    pub fn remove(&mut self, pathname: &str) {
        if let Some(children) = self.parents.get(pathname).cloned() {
            for child in children {
                self.remove_entry(&child);
            }
        }
        self.remove_entry(pathname);
        self.changed = true;
    }

    fn remove_entry(&mut self, pathname: &str) {
        if let Some(entry) = self.entries.remove(pathname) {
            for dirname in entry.parent_dirs() {
                if let Some(ref mut children_set) = self.parents.get_mut(dirname) {
                    children_set.remove(pathname);
                    if children_set.is_empty() {
                        self.parents.remove(dirname);
                    }
                }
            }
        }
    }
    //2
    pub fn add(&mut self, pathname: &str, oid: &str, metadata: &fs::Metadata) {
        let entry = Entry::new(pathname, oid, metadata);
        self.discard_conflicts(&entry);
        self.store_entry(entry);
        self.changed = true;
    }

    pub fn store_entry(&mut self, entry: Entry) {
        self.entries.insert(entry.path.clone(), entry.clone());

        for dirname in entry.parent_dirs() {
            if let Some(ref mut children_set) = self.parents.get_mut(dirname) {
                children_set.insert(entry.path.clone());
            } else {
                let mut h = HashSet::new();
                h.insert(entry.path.clone());
                self.parents.insert(dirname.to_string(), h);
            }
        }
    }
    //3
    pub fn load_for_update(&mut self) -> Result<(), std::io::Error> {
        self.lockfile.hold_for_update()?;
        self.load()?;

        Ok(())
    }

    fn clear(&mut self) {
        self.entries = BTreeMap::new();
        self.hasher = None;
        self.parents = HashMap::new();
        self.changed = false;
    }

    fn open_index_file(&self) -> Option<File> {
        if self.pathname.exists() {
            OpenOptions::new()
                .read(true)
                .open(self.pathname.clone())
                .ok()
        } else {
            None
        }
    }

    fn read_header(checksum: &mut Checksum<File>) -> usize {
        let data = checksum
            .read(HEADER_SIZE)
            .expect("could not read checksum header");
        let signature = str::from_utf8(&data[0..4]).expect("invalid signature");
        let version = u32::from_be_bytes(data[4..8].try_into().unwrap());
        let count = u32::from_be_bytes(data[8..12].try_into().unwrap());

        if signature != "DIRC" {
            panic!("Signature: expected 'DIRC', but found {}", signature);
        }

        if version != 2 {
            panic!("Version: expected '2', but found {}", version);
        }

        count as usize
    }

    fn read_entries(
        &mut self,
        checksum: &mut Checksum<File>,
        count: usize,
    ) -> Result<(), std::io::Error> {
        for _i in 0..count {
            let mut entry = checksum.read(MIN_ENTRY_SIZE)?;
            while entry.last().unwrap() != &0u8 {
                entry.extend_from_slice(&checksum.read(8)?);
            }

            self.store_entry(Entry::parse(&entry)?);
        }

        Ok(())
    }

    pub fn load(&mut self) -> Result<(), std::io::Error> {
        self.clear();
        if let Some(file) = self.open_index_file() {
            let mut reader = Checksum::new(file);
            let count = Index::read_header(&mut reader);
            self.read_entries(&mut reader, count)?;
            reader.verify_checksum()?;
        }

        Ok(())
    }
    //1
    pub fn release_lock(&mut self) -> Result<(), std::io::Error> {
        self.lockfile.rollback()
    }
}
